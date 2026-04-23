use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

pub const CONFIG_VERSION: u32 = 1;
const DEFAULT_LANGUAGES: &[&str] = &[
    "python",
    "rust",
    "javascript",
    "typescript",
    "go",
    "html",
    "css",
    "yaml",
    "shell",
    "json",
    "markdown",
    "toml",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub version: u32,
    pub foundation: FoundationConfig,
    pub languages: LanguagesConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FoundationConfig {
    #[serde(alias = "pinned_paths")]
    pub pinned: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LanguagesConfig {
    pub enabled: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    InvalidToml(String),
    UnsupportedVersion(u32),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidToml(detail) => write!(f, "invalid TOML: {detail}"),
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported config version: {version}")
            }
        }
    }
}

impl Error for ConfigError {}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: default_version(),
            foundation: FoundationConfig::default(),
            languages: LanguagesConfig::default(),
        }
    }
}

impl Default for LanguagesConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_LANGUAGES
                .iter()
                .map(|item| item.to_string())
                .collect(),
        }
    }
}

impl Config {
    pub fn parse(input: &str) -> Result<Self, ConfigError> {
        let config: Self =
            toml::from_str(input).map_err(|error| ConfigError::InvalidToml(error.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    pub fn render(&self) -> Result<String, ConfigError> {
        self.validate()?;

        Ok(format!(
            "version = {}\n\n[foundation]\npinned = {}\n\n[languages]\nenabled = {}\n",
            self.version,
            render_string_array(&self.foundation.pinned),
            render_string_array(&self.languages.enabled)
        ))
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.version != CONFIG_VERSION {
            return Err(ConfigError::UnsupportedVersion(self.version));
        }

        Ok(())
    }
}

fn render_string_array(items: &[String]) -> String {
    let rendered = items
        .iter()
        .map(|item| format!("\"{}\"", item))
        .collect::<Vec<_>>()
        .join(", ");

    format!("[{rendered}]")
}

fn default_version() -> u32 {
    CONFIG_VERSION
}

#[cfg(test)]
mod tests {
    use super::{Config, CONFIG_VERSION};

    #[test]
    fn config_default_round_trip() {
        let config = Config::default();
        let rendered = config.render().expect("default config should render");

        assert_eq!(
            rendered,
            "version = 1\n\n[foundation]\npinned = []\n\n[languages]\nenabled = [\"python\", \"rust\", \"javascript\", \"typescript\", \"go\", \"html\", \"css\", \"yaml\", \"shell\", \"json\", \"markdown\", \"toml\"]\n"
        );
        assert_eq!(
            Config::parse(&rendered).expect("rendered config should parse"),
            config
        );
    }

    #[test]
    fn config_round_trip_with_pinned_paths() {
        let input =
            "version = 1\n\n[foundation]\npinned = [\"AGENTS.md\", \"CLAUDE.md\"]\n\n[languages]\nenabled = [\"python\", \"rust\", \"javascript\", \"typescript\", \"go\", \"html\", \"css\", \"yaml\", \"shell\", \"json\", \"markdown\", \"toml\"]\n";

        let config = Config::parse(input).expect("config should parse");

        assert_eq!(config.version, CONFIG_VERSION);
        assert_eq!(
            config.foundation.pinned,
            vec!["AGENTS.md".to_string(), "CLAUDE.md".to_string()]
        );
        assert_eq!(
            config.languages.enabled,
            vec![
                "python".to_string(),
                "rust".to_string(),
                "javascript".to_string(),
                "typescript".to_string(),
                "go".to_string(),
                "html".to_string(),
                "css".to_string(),
                "yaml".to_string(),
                "shell".to_string(),
                "json".to_string(),
                "markdown".to_string(),
                "toml".to_string()
            ]
        );
        assert_eq!(config.render().expect("config should render"), input);
    }

    #[test]
    fn parse_accepts_valid_toml_with_comments_and_trailing_comma() {
        let input = r#"
            # existing repo config
            version = 1

            [foundation]
            pinned = [
              'AGENTS.md',
              "CLAUDE.md",
            ]

            [languages]
            enabled = ["python", "rust"]
        "#;

        let config = Config::parse(input).expect("config should parse");

        assert_eq!(config.version, CONFIG_VERSION);
        assert_eq!(
            config.foundation.pinned,
            vec!["AGENTS.md".to_string(), "CLAUDE.md".to_string()]
        );
        assert_eq!(
            config.languages.enabled,
            vec!["python".to_string(), "rust".to_string()]
        );
    }

    #[test]
    fn parse_defaults_missing_version_and_foundation() {
        let config = Config::parse("").expect("empty config should parse");

        assert_eq!(config, Config::default());
    }

    #[test]
    fn parse_accepts_legacy_pinned_paths_alias() {
        let input = "version = 1\n\n[foundation]\npinned_paths = [\"AGENTS.md\"]\n";

        let config = Config::parse(input).expect("legacy config should parse");

        assert_eq!(config.foundation.pinned, vec!["AGENTS.md".to_string()]);
    }
}
