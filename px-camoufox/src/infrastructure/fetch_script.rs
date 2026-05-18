//! JS fragment that runs an in-page `fetch` inside Camoufox and posts
//! the response back to fantoccini via `execute_async`'s callback.

use px_errors::AppError;
use px_pipeline::FetchRequest;

pub(crate) fn build(req: &FetchRequest) -> Result<String, AppError> {
    let method = req.method().to_string();
    let headers_json = serde_json::to_string(&req.headers)
        .map_err(|e| AppError::InternalError(format!("encode headers: {e}")))?;
    let body_json = serde_json::to_string(req.body.as_deref().unwrap_or(""))
        .map_err(|e| AppError::InternalError(format!("encode body: {e}")))?;
    let url_json = serde_json::to_string(&req.url)
        .map_err(|e| AppError::InternalError(format!("encode url: {e}")))?;
    let method_json = serde_json::to_string(&method)
        .map_err(|e| AppError::InternalError(format!("encode method: {e}")))?;
    Ok(format!(
        r#"
        const cb = arguments[arguments.length - 1];
        const opts = {{ method: {method_json}, headers: {headers_json}, credentials: 'include' }};
        const body = {body_json};
        if (body !== '' && {method_json} !== 'GET') opts.body = body;
        fetch({url_json}, opts)
          .then(async (r) => {{
            const text = await r.text();
            const hdrs = {{}};
            r.headers.forEach((v, k) => {{ hdrs[k] = v; }});
            cb({{ status: r.status, headers: hdrs, body: text }});
          }})
          .catch((e) => cb({{ status: 0, headers: {{}}, body: String(e) }}));
        "#
    ))
}
