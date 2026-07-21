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
    // Setup logging. The guard must live for the duration of main — dropping it early
    // shuts down the background writer thread and silently discards all log messages.
    let _log_guard;
    if let Some(log_dir) = dirs::data_local_dir() {
        let log_path = log_dir.join("ssm-connect").join("logs");
        std::fs::create_dir_all(&log_path).ok();

        let file_appender = RollingFileAppender::new(Rotation::DAILY, log_path, "ssm-connect.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        _log_guard = Some(guard);

        let subscriber = tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            ))
            .with(tracing_subscriber::fmt::layer().with_writer(non_blocking));

        tracing::subscriber::set_global_default(subscriber).ok();
    } else {
        _log_guard = None;
    }

    tracing::info!("ssm-connect starting");

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

                // Wait for the SSM agent to come online (up to 3 attempts, 3s apart).
                // This handles the window between an instance reaching "running" state
                // and the SSM agent registering with the service.
                const MAX_AGENT_ATTEMPTS: u32 = 3;
                let mut agent_ready = false;
                for attempt in 1..=MAX_AGENT_ATTEMPTS {
                    if ssm.is_agent_online(&instance_id).await {
                        agent_ready = true;
                        break;
                    }
                    if attempt < MAX_AGENT_ATTEMPTS {
                        println!(
                            "SSM agent not ready yet, retrying in 3s... ({}/{})",
                            attempt, MAX_AGENT_ATTEMPTS
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    }
                }
                if !agent_ready {
                    eprintln!(
                        "Error: SSM agent on {} is not responding after {} attempts.",
                        instance_id, MAX_AGENT_ATTEMPTS
                    );
                    eprintln!("The agent may still be initialising. Try again in a few seconds.");
                    std::process::exit(1);
                }

                // Spawn port forwarding processes before exec'ing the interactive session
                let enabled_forwards = app.settings.enabled_forwards_for(&instance_id);
                if !enabled_forwards.is_empty() {
                    println!("Starting port forwarding:");
                    for rule in &enabled_forwards {
                        println!("  {} : localhost:{} → instance:{}", rule.alias, rule.local_port, rule.remote_port);
                        tracing::info!(alias = %rule.alias, local_port = rule.local_port, remote_port = rule.remote_port, "starting port forward");
                        if let Err(e) = ssm.start_port_forwarding(&instance_id, rule, app.credentials.as_ref()) {
                            tracing::error!(alias = %rule.alias, error = %e, "failed to spawn port forward process");
                            eprintln!("Warning: failed to start port forwarding for {}: {}", rule.alias, e);
                        }
                    }
                    println!("(Forwarding processes will run until SSM sessions time out)");
                }

                // Get shell preference from settings
                let shell = app.settings.default_shell.as_str();

                // Launch SSM session with credentials and auto-commands
                match ssm.launch_session(&instance_id, shell, app.credentials.as_ref(), &app.settings.auto_execute_commands) {
                    Ok(_) => {}
                    Err(e) => {
                        let error_msg = e.to_string();
                        if error_msg.contains("TargetNotConnected") {
                            eprintln!("Error: Instance is running but SSM agent is not ready yet.");
                            eprintln!("This usually takes 30-60 seconds after instance starts.");
                            eprintln!("Please wait a moment and try connecting again.");
                        } else {
                            eprintln!("Error connecting to instance: {}", e);
                        }
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
