use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct CamoufoxConfig {
    pub camoufox_bin: PathBuf,
    pub geckodriver_bin: PathBuf,
    pub headless: bool,
    pub max_concurrent: usize,
    pub navigate_timeout: Duration,
    pub locale: String,
    pub timezone: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CamoufoxConfigError {
    CamoufoxNotFound(PathBuf),
    GeckodriverNotFound(PathBuf),
}

impl fmt::Display for CamoufoxConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CamoufoxNotFound(p) => write!(
                f,
                "Camoufox binary not at {}; set PX_CAMOUFOX_BIN or install via `pip install camoufox && python -m camoufox fetch`",
                p.display()
            ),
            Self::GeckodriverNotFound(p) => write!(
                f,
                "geckodriver not at {}; set PX_GECKODRIVER_BIN or install from https://github.com/mozilla/geckodriver/releases",
                p.display()
            ),
        }
    }
}

impl std::error::Error for CamoufoxConfigError {}

impl CamoufoxConfig {
    pub fn from_env() -> Self {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        let camoufox_bin = std::env::var_os("PX_CAMOUFOX_BIN")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".cache/camoufox/camoufox"));
        let geckodriver_bin = std::env::var_os("PX_GECKODRIVER_BIN")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/bin/geckodriver"));
        let locale = std::env::var("PX_CAMOUFOX_LOCALE").unwrap_or_else(|_| "en-US".into());
        Self {
            camoufox_bin,
            geckodriver_bin,
            headless: true,
            max_concurrent: 2,
            navigate_timeout: Duration::from_secs(45),
            locale,
            timezone: None,
        }
    }

    pub fn validate(&self) -> Result<(), CamoufoxConfigError> {
        if !self.camoufox_bin.exists() {
            return Err(CamoufoxConfigError::CamoufoxNotFound(
                self.camoufox_bin.clone(),
            ));
        }
        if !self.geckodriver_bin.exists() {
            return Err(CamoufoxConfigError::GeckodriverNotFound(
                self.geckodriver_bin.clone(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn from_env_uses_defaults_when_unset() {
        let cfg = CamoufoxConfig::from_env();
        assert!(cfg.camoufox_bin.to_string_lossy().contains("camoufox"));
        assert!(
            cfg.geckodriver_bin
                .to_string_lossy()
                .contains("geckodriver")
        );
        assert!(cfg.headless);
    }

    #[test]
    fn validate_rejects_missing_camoufox() {
        let cfg = CamoufoxConfig {
            camoufox_bin: PathBuf::from("/nowhere/camoufox"),
            geckodriver_bin: PathBuf::from("/nowhere/geckodriver"),
            headless: true,
            max_concurrent: 1,
            navigate_timeout: Duration::from_secs(30),
            locale: "en-US".into(),
            timezone: None,
        };
        assert!(matches!(
            cfg.validate(),
            Err(CamoufoxConfigError::CamoufoxNotFound(_))
        ));
    }
}
