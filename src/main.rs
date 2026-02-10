mod app;
mod aws;
mod config;
mod error;
mod events;
mod ui;

use app::App;
use aws::{CredentialManager, SsmManager};
use clap::Parser;
use config::Settings;
use crossterm::event::{DisableBracketedPaste, EnableBracketedPaste};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use error::Result;
use events::EventHandler;
use std::io::{self, stdout};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::layer::SubscriberExt;

/// A modern terminal UI for connecting to EC2 instances via AWS Systems Manager (SSM)
#[derive(Parser)]
#[command(name = "ssm-connect")]
#[command(version = "1.0.0")]
#[command(about = "Connect to EC2 instances via AWS Systems Manager")]
#[command(long_about = "A modern terminal UI for connecting to EC2 instances via AWS Systems Manager (SSM).

Features:
  • Automatic AWS credential discovery with manual fallback
  • Visual instance browser with real-time status
  • Quick instance starting for stopped instances
  • Easy region switching
  • Keyboard-driven interface
  • Auto-execute commands on connection

Configuration:
  Config file: ~/.config/ssm-connect/config.json
  Logs: ~/.local/share/ssm-connect/logs/

Keyboard Shortcuts:
  ↑/↓ or j/k - Navigate
  Enter      - Connect to instance
  s          - Start stopped instance
  r          - Change region
  c          - Configure auto-execute commands
  ?          - Toggle help
  q/Esc      - Quit
")]
struct Cli {}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments (handles --help and --version automatically)
    let _cli = Cli::parse();
    // Setup logging
    if let Some(log_dir) = dirs::data_local_dir() {
        let log_path = log_dir.join("ssm-connect").join("logs");
        std::fs::create_dir_all(&log_path).ok();

        let file_appender = RollingFileAppender::new(Rotation::DAILY, log_path, "ssm-connect.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        let subscriber = tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            ))
            .with(tracing_subscriber::fmt::layer().with_writer(non_blocking));

        tracing::subscriber::set_global_default(subscriber).ok();
    }

    // Install panic handler that restores terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
        let _ = stdout().execute(DisableBracketedPaste);
        original_hook(panic_info);
    }));

    // Load settings
    let settings = Settings::load().unwrap_or_default();

    // Determine initial region from AWS config (respects ~/.aws/config)
    let sdk_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let initial_region = sdk_config
        .region()
        .map(|r| r.as_ref().to_string())
        .unwrap_or_else(|| settings.default_region.clone());

    // Create app with settings
    let app = App::new(initial_region, settings);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(EnableBracketedPaste)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let terminal = ratatui::Terminal::new(backend)?;

    // Create event handler
    let event_handler = EventHandler::new();

    // Run event loop
    let app = event_handler.run(app, terminal).await?;

    // Restore terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    io::stdout().execute(DisableBracketedPaste)?;

    // Check if we need to connect to an instance
    if let Some(instance_id) = app.pending_connection {
        println!("Connecting to instance: {}", instance_id);

        // Debug: Show auto-execute commands if any
        if !app.settings.auto_execute_commands.is_empty() {
            println!("Auto-execute commands configured:");
            for (i, cmd) in app.settings.auto_execute_commands.iter().enumerate() {
                println!("  {}. {}", i + 1, cmd);
            }
        }

        // Rebuild AWS config from app credentials and region
        let config_result = if let Some(creds) = &app.credentials {
            if creds.access_key_id == "default" {
                // Using default credentials
                CredentialManager::discover_credentials().await
            } else {
                // Using manual credentials
                CredentialManager::build_config(creds, &app.region)
                    .await
                    .map(Some)
            }
        } else {
            // Fallback to default credentials
            CredentialManager::discover_credentials().await
        };

        match config_result {
            Ok(Some(config)) => {
                // Create SSM manager
                let ssm = SsmManager::new(&config, app.region.clone());

                // Get shell preference from settings
                let shell = app.settings.default_shell.as_str();

                // Launch SSM session with credentials and auto-commands
                match ssm.launch_session(&instance_id, shell, app.credentials.as_ref(), &app.settings.auto_execute_commands) {
                    Ok(_) => {
                        println!("Session ended successfully.");
                    }
                    Err(e) => {
                        eprintln!("Error connecting to instance: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            Ok(None) => {
                eprintln!("Error: No AWS credentials available");
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("Error loading AWS credentials: {}", e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
