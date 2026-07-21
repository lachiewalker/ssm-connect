use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForwardingRule {
    pub alias: String,
    pub local_port: u16,
    pub remote_port: u16,
}

fn default_port_forwarding_rules() -> Vec<PortForwardingRule> {
    vec![
        PortForwardingRule { alias: "TensorBoard".to_string(), local_port: 6006, remote_port: 6006 },
        PortForwardingRule { alias: "Jupyter".to_string(), local_port: 8888, remote_port: 8888 },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub default_region: String,
    pub default_shell: String,
    pub auto_refresh_interval_seconds: u64,
    pub theme: String,
    pub auto_execute_commands: Vec<String>,
    #[serde(default = "default_port_forwarding_rules")]
    pub port_forwarding_rules: Vec<PortForwardingRule>,
    #[serde(default)]
    pub instance_port_forwards: HashMap<String, Vec<String>>, // instance_id → enabled rule aliases
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_region: "us-east-1".to_string(),
            default_shell: "bash".to_string(),
            auto_refresh_interval_seconds: 30,
            theme: "dark".to_string(),
            auto_execute_commands: Vec::new(),
            port_forwarding_rules: default_port_forwarding_rules(),
            instance_port_forwards: HashMap::new(),
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

    pub fn enabled_forwards_for(&self, instance_id: &str) -> Vec<&PortForwardingRule> {
        let enabled_aliases = match self.instance_port_forwards.get(instance_id) {
            Some(aliases) => aliases,
            None => return vec![],
        };
        self.port_forwarding_rules
            .iter()
            .filter(|r| enabled_aliases.contains(&r.alias))
            .collect()
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
        assert_eq!(settings.port_forwarding_rules.len(), 2);
        assert!(settings.instance_port_forwards.is_empty());
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

    #[test]
    fn test_enabled_forwards_for() {
        let mut settings = Settings::default();
        settings.instance_port_forwards.insert(
            "i-123".to_string(),
            vec!["TensorBoard".to_string()],
        );
        let forwards = settings.enabled_forwards_for("i-123");
        assert_eq!(forwards.len(), 1);
        assert_eq!(forwards[0].alias, "TensorBoard");

        let forwards_none = settings.enabled_forwards_for("i-999");
        assert!(forwards_none.is_empty());
    }
}
