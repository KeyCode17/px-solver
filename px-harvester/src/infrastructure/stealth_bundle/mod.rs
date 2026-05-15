pub mod patches {
    pub const NAVIGATOR_WEBDRIVER: &str = include_str!("navigator_webdriver.js");
    pub const NAVIGATOR_LANGUAGES: &str = include_str!("navigator_languages.js");
    pub const NAVIGATOR_PLUGINS: &str = include_str!("navigator_plugins.js");
    pub const CHROME_RUNTIME: &str = include_str!("chrome_runtime.js");
    pub const WEBGL_VENDOR_RENDERER: &str = include_str!("webgl_vendor_renderer.js");
    pub const CANVAS_NOISE: &str = include_str!("canvas_noise.js");
    pub const AUDIO_NOISE: &str = include_str!("audio_noise.js");
    pub const PERMISSIONS_QUERY: &str = include_str!("permissions_query.js");
}
