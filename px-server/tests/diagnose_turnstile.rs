#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Manual Turnstile-click probe.
//!
//! Drives chromiumoxide directly (bypassing the Harvester trait) so we can
//! navigate → locate the Turnstile iframe → CDP-click its center → wait for
//! cf_clearance → extract cookies + html. Pure diagnostic — not wired into
//! production paths yet.
//!
//! Run with:
//!   DIAGNOSE_TURNSTILE=1 DIAGNOSE_HEADLESS=0 cargo test -q -p px-server \
//!     --test diagnose_turnstile -- --ignored --nocapture

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::input::DispatchMouseEventParams;
use chromiumoxide::cdp::browser_protocol::input::DispatchMouseEventType;
use chromiumoxide::cdp::browser_protocol::input::MouseButton;
use futures::StreamExt;
use std::time::Duration;

#[tokio::test]
#[ignore]
async fn diagnose_turnstile_click() {
    if std::env::var("DIAGNOSE_TURNSTILE").ok().as_deref() != Some("1") {
        eprintln!("set DIAGNOSE_TURNSTILE=1 to run");
        return;
    }
    let url =
        std::env::var("DIAGNOSE_URL").unwrap_or_else(|_| "https://www.pedidosya.com.ar/".into());
    let headless = std::env::var("DIAGNOSE_HEADLESS")
        .ok()
        .as_deref()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(true);

    let mut cfg = BrowserConfig::builder()
        .viewport(None)
        .window_size(1280, 800);
    if !headless {
        cfg = cfg.with_head();
    }
    let (mut browser, mut handler) = Browser::launch(cfg.build().expect("browser config"))
        .await
        .expect("launch");
    let _handle = tokio::spawn(async move { while handler.next().await.is_some() {} });

    let page = browser.new_page("about:blank").await.expect("new page");
    page.goto(&url).await.expect("goto");
    eprintln!("[probe] navigated to {url}");

    // Poll for Turnstile iframe up to 20s — CF challenge JS may take time to render it.
    let mut bbox: Option<BBox> = None;
    for attempt in 0..20 {
        tokio::time::sleep(Duration::from_millis(1_000)).await;
        let iframes_json = page
            .evaluate(
                r#"Array.from(document.querySelectorAll('iframe')).map(f => ({
                    src: (f.src || '').substring(0, 80),
                    title: f.title || '',
                    id: f.id || '',
                }))"#,
            )
            .await
            .expect("eval iframes");
        let iframes: Vec<IframeInfo> = iframes_json.into_value().unwrap_or_default();
        eprintln!("[probe] t={}s iframes={}", attempt + 1, iframes.len());
        for f in &iframes {
            eprintln!("    src={:?} title={:?} id={:?}", f.src, f.title, f.id);
        }
        let bbox_eval = page
            .evaluate(
                r#"
                (() => {
                    const f = document.querySelector('iframe[src*="challenges.cloudflare.com"]')
                        || document.querySelector('iframe[title*="urnstile"]')
                        || document.querySelector('iframe[title*="Cloudflare"]');
                    if (!f) return null;
                    const r = f.getBoundingClientRect();
                    return { x: r.x, y: r.y, w: r.width, h: r.height };
                })()
                "#,
            )
            .await
            .expect("eval bbox");
        if let Ok(b) = bbox_eval.into_value::<BBox>() {
            bbox = Some(b);
            break;
        }
    }
    let pre_html = page.content().await.expect("content");
    eprintln!("[probe] pre-click html len = {}", pre_html.len());
    eprintln!("[probe] pre-click title = {:?}", title_of(&pre_html));
    let Some(bbox) = bbox else {
        eprintln!(
            "[probe] turnstile iframe never appeared in 20s — CF stuck in pre-Turnstile phase"
        );
        let _ = browser.close().await;
        return;
    };
    eprintln!("[probe] turnstile iframe bbox = {bbox:?}");

    // CDP mouse click in viewport coords — passes through cross-origin iframes.
    let click_x = bbox.x + 30.0; // checkbox is near left edge of widget
    let click_y = bbox.y + bbox.h / 2.0;
    eprintln!("[probe] clicking at ({click_x}, {click_y})");
    for evt in [
        DispatchMouseEventType::MouseMoved,
        DispatchMouseEventType::MousePressed,
        DispatchMouseEventType::MouseReleased,
    ] {
        let params = DispatchMouseEventParams::builder()
            .x(click_x)
            .y(click_y)
            .button(MouseButton::Left)
            .click_count(1)
            .r#type(evt)
            .build()
            .expect("mouse params");
        page.execute(params).await.expect("dispatch mouse");
        tokio::time::sleep(Duration::from_millis(60)).await;
    }

    // Wait for CF to process the click and (hopefully) redirect or issue cf_clearance.
    tokio::time::sleep(Duration::from_millis(8_000)).await;

    let post_html = page.content().await.expect("post content");
    eprintln!("[probe] post-click html len = {}", post_html.len());
    eprintln!("[probe] post-click title = {:?}", title_of(&post_html));
    let cookies = page.get_cookies().await.expect("cookies");
    eprintln!("[probe] cookies after click: {}", cookies.len());
    for c in &cookies {
        let v: String = c.value.chars().take(30).collect();
        eprintln!(
            "  - {} = {}{}",
            c.name,
            v,
            if c.value.len() > 30 { "..." } else { "" }
        );
    }
    let cleared = cookies.iter().any(|c| c.name == "cf_clearance");
    let has_px3 = cookies.iter().any(|c| c.name == "_px3");
    eprintln!("[probe] cf_clearance present: {cleared}");
    eprintln!("[probe] _px3 present:         {has_px3}");

    let _ = browser.close().await;
}

fn title_of(html: &str) -> Option<&str> {
    let i = html.find("<title>")? + 7;
    let j = html[i..].find("</title>")?;
    Some(&html[i..i + j])
}

#[derive(Debug, serde::Deserialize)]
struct BBox {
    x: f64,
    y: f64,
    #[allow(dead_code)]
    w: f64,
    h: f64,
}

#[derive(Debug, Default, serde::Deserialize)]
struct IframeInfo {
    #[serde(default)]
    src: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    id: String,
}
