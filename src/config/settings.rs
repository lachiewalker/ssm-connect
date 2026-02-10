use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub default_region: String,
    pub default_shell: String,
    pub auto_refresh_interval_seconds: u64,
    pub theme: String,
    pub auto_execute_commands: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_region: "us-east-1".to_string(),
            default_shell: "bash".to_string(),
            auto_refresh_interval_seconds: 30,
            theme: "dark".to_string(),
            auto_execute_commands: Vec::new(),
        }
    }
}

impl Settings {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("ssm-connect").join("config.json"))
    }

    pub fn load() -> Option<Self> {
        let path = Self::config_path()?;
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "Could not find config directory")
        })?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;

        // Write and explicitly sync to ensure changes are flushed
        use std::fs::File;
        use std::io::Write;
        let mut file = File::create(&path)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default() {
        let settings = Settings::default();
        assert_eq!(settings.default_region, "us-east-1");
        assert_eq!(settings.default_shell, "bash");
        assert_eq!(settings.auto_refresh_interval_seconds, 30);
        assert_eq!(settings.theme, "dark");
        assert!(settings.auto_execute_commands.is_empty());
    }

    #[test]
    fn test_settings_serialization() {
        let mut settings = Settings::default();
        settings.auto_execute_commands = vec![
            "cd /tmp".to_string(),
            "echo 'Hello'".to_string(),
        ];

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: Settings = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.auto_execute_commands.len(), 2);
        assert_eq!(deserialized.auto_execute_commands[0], "cd /tmp");
        assert_eq!(deserialized.auto_execute_commands[1], "echo 'Hello'");
    }
}
