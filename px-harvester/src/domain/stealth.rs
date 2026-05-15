use crate::infrastructure::stealth_bundle::patches;

#[derive(Debug, Clone)]
pub struct StealthBundle {
    patches: Vec<&'static str>,
}

impl StealthBundle {
    pub fn empty() -> Self {
        Self {
            patches: Vec::new(),
        }
    }

    pub fn push(mut self, patch: &'static str) -> Self {
        self.patches.push(patch);
        self
    }

    pub fn patches(&self) -> &[&'static str] {
        &self.patches
    }

    pub fn combined(&self) -> String {
        let mut out = String::new();
        for p in &self.patches {
            out.push_str("(function(){");
            out.push_str(p);
            out.push_str("\n})();\n");
        }
        out
    }
}

impl Default for StealthBundle {
    fn default() -> Self {
        default_stealth_bundle()
    }
}

pub fn default_stealth_bundle() -> StealthBundle {
    StealthBundle::empty()
        .push(patches::NAVIGATOR_WEBDRIVER)
        .push(patches::NAVIGATOR_LANGUAGES)
        .push(patches::NAVIGATOR_PLUGINS)
        .push(patches::CHROME_RUNTIME)
        .push(patches::WEBGL_VENDOR_RENDERER)
        .push(patches::CANVAS_NOISE)
        .push(patches::AUDIO_NOISE)
        .push(patches::PERMISSIONS_QUERY)
}
