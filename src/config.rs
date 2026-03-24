use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_repo")]
    pub default_repo: String,
    #[serde(default = "default_base_branch")]
    pub default_base_branch: String,
    #[serde(default = "default_command")]
    pub default_command: String,
    #[serde(default = "default_master_ratio")]
    pub master_ratio: f64,
    #[serde(default = "default_master_count")]
    pub master_count: usize,
    #[serde(default = "default_layout")]
    pub default_layout: String,
    #[serde(default = "default_border_style")]
    pub border_style: String,
    #[serde(default)]
    pub color_scheme: ColorScheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    #[serde(default = "default_running_color")]
    pub running: String,
    #[serde(default = "default_waiting_color")]
    pub waiting: String,
    #[serde(default = "default_done_color")]
    pub done: String,
    #[serde(default = "default_paused_color")]
    pub paused: String,
    #[serde(default = "default_focused_color")]
    pub focused: String,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            running: "yellow".into(),
            waiting: "green".into(),
            done: "gray".into(),
            paused: "blue".into(),
            focused: "white".into(),
        }
    }
}

fn default_repo() -> String {
    ".".into()
}
fn default_base_branch() -> String {
    "main".into()
}
fn default_command() -> String {
    "claude".into()
}
fn default_master_ratio() -> f64 {
    0.55
}
fn default_master_count() -> usize {
    1
}
fn default_layout() -> String {
    "master_stack".into()
}
fn default_border_style() -> String {
    "rounded".into()
}
fn default_running_color() -> String {
    "yellow".into()
}
fn default_waiting_color() -> String {
    "green".into()
}
fn default_done_color() -> String {
    "gray".into()
}
fn default_paused_color() -> String {
    "blue".into()
}
fn default_focused_color() -> String {
    "white".into()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_repo: default_repo(),
            default_base_branch: default_base_branch(),
            default_command: default_command(),
            master_ratio: default_master_ratio(),
            master_count: default_master_count(),
            default_layout: default_layout(),
            border_style: default_border_style(),
            color_scheme: ColorScheme::default(),
        }
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("claude-dtm")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn sessions_path() -> PathBuf {
        Self::config_dir().join("sessions.json")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let contents = std::fs::read_to_string(&path)?;
            let config: Config = serde_json::from_str(&contents)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)?;
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(Self::config_path(), contents)?;
        Ok(())
    }

    pub fn resolve_repo_path(&self, cli_repo: Option<&str>) -> PathBuf {
        let repo = cli_repo.unwrap_or(&self.default_repo);
        let path = PathBuf::from(shellexpand(repo));
        if path.is_absolute() {
            path
        } else {
            std::env::current_dir().unwrap_or_default().join(path)
        }
    }
}

fn shellexpand(s: &str) -> String {
    if let Some(rest) = s.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest).to_string_lossy().into_owned();
        }
    }
    s.to_string()
}
