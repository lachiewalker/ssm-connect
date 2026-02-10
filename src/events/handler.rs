use crate::app::{App, Message, Screen, SettingsMode, SettingsScreenState};
use crate::aws::{CredentialManager, Ec2Manager};
use crate::config::Settings;
use crate::events::input::handle_key_event;
use crate::error::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use futures::stream::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc;
use tui_textarea::TextArea;

pub struct EventHandler {
    tx: mpsc::UnboundedSender<Message>,
    rx: mpsc::UnboundedReceiver<Message>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self { tx, rx }
    }

    #[allow(dead_code)]
    pub fn sender(&self) -> mpsc::UnboundedSender<Message> {
        self.tx.clone()
    }

    pub async fn run(
        mut self,
        mut app: App,
        mut terminal: ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> Result<App> {
        let _settings = Settings::load().unwrap_or_default();
        let mut event_stream = EventStream::new();
        let mut tick_interval = tokio::time::interval(Duration::from_millis(250));
        let mut render_interval = tokio::time::interval(Duration::from_millis(16)); // ~60 FPS

        // Try to discover and validate credentials on startup
        let tx = self.tx.clone();
        tokio::spawn(async move {
            if let Ok(Some(config)) = CredentialManager::discover_credentials().await {
                if let Ok(_identity) = CredentialManager::validate_credentials(&config).await {
                    // Extract credentials - note: we don't actually store them, just validate
                    let _ = tx.send(Message::UseDefaultCredentials);
                }
            }
        });

        loop {
            tokio::select! {
                // Handle crossterm events
                Some(Ok(event)) = event_stream.next() => {
                    match event {
                        Event::Paste(data) => {
                            if let Screen::CredentialInput(ref mut state) = app.screen {
                                // Check if this looks like AWS credential export format
                                let is_aws_format = (data.contains('\n') || data.contains('\r'))
                                                    && data.contains("export")
                                                    && data.len() > 100;

                                if is_aws_format {
                                    // Normalize line endings: replace \r\n with \n, then \r with \n
                                    let normalized = data.replace("\r\n", "\n").replace('\r', "\n");

                                    // Try to parse as AWS credentials
                                    match crate::aws::parse_aws_credentials(&normalized) {
                                        Some(parsed) if parsed.access_key.is_some() && parsed.secret_key.is_some() => {
                                            // Successfully parsed! Populate fields directly
                                            let access = parsed.access_key.unwrap();
                                            let secret = parsed.secret_key.unwrap();

                                            // Replace field contents
                                            state.access_key = TextArea::default();
                                            state.access_key.insert_str(&access);
                                            state.access_key.set_placeholder_text("AWS_ACCESS_KEY_ID");

                                            state.secret_key = TextArea::default();
                                            state.secret_key.insert_str(&secret);
                                            state.secret_key.set_placeholder_text("AWS_SECRET_ACCESS_KEY");
                                            state.secret_key.set_mask_char('*');

                                            if let Some(token) = parsed.session_token {
                                                state.session_token = TextArea::default();
                                                state.session_token.insert_str(&token);
                                                state.session_token.set_placeholder_text("AWS_SESSION_TOKEN (optional)");
                                            }

                                            state.hint_message = Some("✓ Credentials parsed and filled! Press Enter to submit.".to_string());
                                        }
                                        _ => {
                                            // Parsing failed
                                            state.hint_message = Some("⚠ Could not parse AWS credentials. Please check format or enter manually.".to_string());
                                        }
                                    }
                                } else {
                                    // Not AWS format, insert into current field
                                    state.current_field_mut().insert_str(&data);
                                }
                            }
                        }
                        Event::Key(key) if key.kind == KeyEventKind::Press => {
                            // Check if this is a command key first
                            if let Some(msg) = handle_key_event(&app, key) {
                                let _ = self.tx.send(msg);
                            } else {
                                // Handle text input for credential screen
                                if let Screen::CredentialInput(ref mut state) = app.screen {
                                    match key.code {
                                        KeyCode::Char(c) => {
                                            state.current_field_mut().insert_char(c);
                                        }
                                        KeyCode::Backspace => {
                                            state.current_field_mut().delete_char();
                                        }
                                        _ => {
                                            // Let TextArea handle other keys
                                            state.current_field_mut().input(key);
                                        }
                                    }
                                } else if let Screen::Settings(ref mut state) = app.screen {
                                    if state.mode == SettingsMode::Edit {
                                        if let Some(ref mut textarea) = state.edit_textarea {
                                            match key.code {
                                                KeyCode::Char(c) => {
                                                    textarea.insert_char(c);
                                                }
                                                KeyCode::Backspace => {
                                                    textarea.delete_char();
                                                }
                                                _ => {
                                                    textarea.input(key);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // Handle messages from async tasks
                Some(msg) = self.rx.recv() => {
                    self.handle_message(&mut app, msg, &_settings).await;
                }

                // Periodic tick for state updates
                _ = tick_interval.tick() => {
                    // Could add periodic refresh logic here
                }

                // Render interval
                _ = render_interval.tick() => {
                    terminal.draw(|f| crate::ui::render::render(f, &mut app))?;
                }
            }

            if app.should_quit {
                break;
            }
        }

        Ok(app)
    }

    async fn handle_message(&self, app: &mut App, msg: Message, _settings: &Settings) {
        match &msg {
            Message::Select => {
                if let Screen::RegionSelection { selected } = app.screen {
                    let regions = crate::app::get_aws_regions();
                    if selected < regions.len() {
                        let new_region = regions[selected].to_string();
                        let _ = self.tx.send(Message::RegionChanged(new_region));
                    }
                }
            }
            Message::SubmitCredentials => {
                if let Screen::CredentialInput(ref mut state) = app.screen {
                    if let Some(creds) = state.get_credentials() {
                        // Validate credentials with AWS
                        app.loading = crate::app::LoadingState::ValidatingCredentials;
                        let tx = self.tx.clone();
                        let region = app.region.clone();

                        tokio::spawn(async move {
                            let config = CredentialManager::build_config(&creds, &region).await;
                            match config {
                                Ok(config) => {
                                    match CredentialManager::validate_credentials(&config).await {
                                        Ok(_) => {
                                            let _ = tx.send(Message::CredentialsValidated(creds, region));
                                        }
                                        Err(e) => {
                                            let _ = tx.send(Message::CredentialValidationFailed(
                                                e.to_string(),
                                            ));
                                        }
                                    }
                                }
                                Err(e) => {
                                    let _ = tx.send(Message::CredentialValidationFailed(
                                        e.to_string(),
                                    ));
                                }
                            }
                        });
                    }
                    // If None, hint message will be shown automatically
                }
            }
            Message::UseDefaultCredentials => {
                let tx = self.tx.clone();
                tokio::spawn(async move {
                    match CredentialManager::discover_credentials().await {
                        Ok(Some(config)) => {
                            match CredentialManager::validate_credentials(&config).await {
                                Ok(_) => {
                                    let region = config
                                        .region()
                                        .map(|r| r.to_string())
                                        .unwrap_or_else(|| "us-east-1".to_string());

                                    // Create dummy credentials for validation
                                    let creds = crate::aws::types::AwsCredentials {
                                        access_key_id: "default".to_string(),
                                        secret_access_key: "default".to_string(),
                                        session_token: None,
                                    };

                                    let _ = tx.send(Message::CredentialsValidated(creds, region));
                                }
                                Err(e) => {
                                    let _ = tx.send(Message::CredentialValidationFailed(
                                        e.to_string(),
                                    ));
                                }
                            }
                        }
                        Ok(None) => {
                            let _ = tx.send(Message::CredentialValidationFailed(
                                "No default credentials found".to_string(),
                            ));
                        }
                        Err(e) => {
                            let _ = tx.send(Message::CredentialValidationFailed(e.to_string()));
                        }
                    }
                });
            }
            Message::CredentialsValidated(creds, region) => {
                // Store credentials and load instances
                let creds = creds.clone();
                let region = region.clone();
                app.credentials = Some(creds.clone());
                app.region = region.clone();
                app.screen = Screen::InstanceList;
                app.loading = crate::app::LoadingState::LoadingInstances;

                // Load instances in background
                let tx = self.tx.clone();
                tokio::spawn(async move {
                    let config = if creds.access_key_id == "default" {
                        CredentialManager::discover_credentials().await.ok().flatten()
                    } else {
                        CredentialManager::build_config(&creds, &region).await.ok()
                    };

                    if let Some(config) = config {
                        let ec2 = Ec2Manager::new(&config);
                        match ec2.list_instances().await {
                            Ok(instances) => {
                                let _ = tx.send(Message::InstancesLoaded(instances));
                            }
                            Err(e) => {
                                let _ = tx.send(Message::Error(format!(
                                    "Failed to load instances: {}",
                                    e
                                )));
                            }
                        }
                    }
                });
            }
            Message::StartInstance => {
                if let Some(instance) = app.selected_instance() {
                    let instance_id = instance.id.clone();
                    let creds = app.credentials.clone();
                    let region = app.region.clone();
                    let tx = self.tx.clone();

                    if instance.state != crate::aws::types::InstanceState::Stopped {
                        let _ = tx.send(Message::Error(
                            "Instance must be stopped to start it".to_string(),
                        ));
                        return;
                    }

                    tokio::spawn(async move {
                        if let Some(creds) = creds {
                            let config = if creds.access_key_id == "default" {
                                CredentialManager::discover_credentials()
                                    .await
                                    .ok()
                                    .flatten()
                            } else {
                                CredentialManager::build_config(&creds, &region).await.ok()
                            };

                            if let Some(config) = config {
                                let ec2 = Ec2Manager::new(&config);

                                match ec2.start_instance(&instance_id).await {
                                    Ok(_) => {
                                        let _ = tx.send(Message::InstanceStarted(instance_id));
                                        // Reload instances after a delay
                                        tokio::time::sleep(Duration::from_secs(2)).await;
                                        let _ = tx.send(Message::UseDefaultCredentials);
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Message::Error(format!(
                                            "Failed to start instance: {}",
                                            e
                                        )));
                                    }
                                }
                            }
                        }
                    });
                }
            }
            Message::ConnectToInstance => {
                if let Some(instance) = app.selected_instance() {
                    use crate::aws::types::InstanceState;

                    // Validate instance is running
                    if instance.state != InstanceState::Running {
                        let _ = self.tx.send(Message::Error(format!(
                            "Instance must be running. Current state: {}",
                            instance.state.as_str()
                        )));
                        return;
                    }

                    // Store the instance ID for connection after TUI exits
                    // If SSM is not available, AWS CLI will give a proper error
                    app.pending_connection = Some(instance.id.clone());
                    app.should_quit = true;
                }
            }
            Message::RegionChanged(new_region) => {
                let new_region = new_region.clone();
                app.region = new_region.clone();
                app.instances.clear();
                app.selected_instance = None;
                app.loading = crate::app::LoadingState::LoadingInstances;
                app.screen = Screen::InstanceList;

                // Reload instances for new region
                if let Some(creds) = &app.credentials {
                    let tx = self.tx.clone();
                    let creds_clone = creds.clone();
                    tokio::spawn(async move {
                        let config = if creds_clone.access_key_id == "default" {
                            CredentialManager::discover_credentials().await.ok().flatten()
                        } else {
                            CredentialManager::build_config(&creds_clone, &new_region)
                                .await
                                .ok()
                        };

                        if let Some(config) = config {
                            let ec2 = Ec2Manager::new(&config);
                            match ec2.list_instances().await {
                                Ok(instances) => {
                                    let _ = tx.send(Message::InstancesLoaded(instances));
                                }
                                Err(e) => {
                                    let _ = tx.send(Message::Error(format!(
                                        "Failed to load instances: {}",
                                        e
                                    )));
                                }
                            }
                        }
                    });
                }
            }
            Message::OpenSettings => {
                app.screen = Screen::Settings(SettingsScreenState {
                    mode: SettingsMode::List,
                    selected_command: 0,
                    edit_textarea: None,
                    edit_index: None,
                });
                // Ensure selected_command is valid
                if let Screen::Settings(ref mut state) = app.screen {
                    if app.settings.auto_execute_commands.is_empty() {
                        state.selected_command = 0;
                    } else if state.selected_command >= app.settings.auto_execute_commands.len() {
                        state.selected_command = app.settings.auto_execute_commands.len() - 1;
                    }
                }
            }
            Message::AddCommand => {
                if let Screen::Settings(ref mut state) = app.screen {
                    let mut textarea = TextArea::default();
                    textarea.set_block(
                        ratatui::widgets::Block::default()
                            .title("Command")
                            .borders(ratatui::widgets::Borders::ALL)
                    );
                    state.mode = SettingsMode::Edit;
                    state.edit_textarea = Some(textarea);
                    state.edit_index = None;  // Adding new
                }
            }
            Message::EditCommand => {
                if let Screen::Settings(ref mut state) = app.screen {
                    if state.selected_command < app.settings.auto_execute_commands.len() {
                        let command = app.settings.auto_execute_commands[state.selected_command].clone();
                        let mut textarea = TextArea::default();
                        textarea.insert_str(&command);
                        textarea.set_block(
                            ratatui::widgets::Block::default()
                                .title("Command")
                                .borders(ratatui::widgets::Borders::ALL)
                        );
                        state.mode = SettingsMode::Edit;
                        state.edit_textarea = Some(textarea);
                        state.edit_index = Some(state.selected_command);
                    }
                }
            }
            Message::DeleteCommand => {
                if let Screen::Settings(ref mut state) = app.screen {
                    let selected = state.selected_command;
                    if selected < app.settings.auto_execute_commands.len() {
                        // Update in-memory settings
                        app.settings.auto_execute_commands.remove(selected);

                        // Save to disk
                        match app.settings.save() {
                            Ok(_) => {
                                // Adjust selected_command if necessary
                                if !app.settings.auto_execute_commands.is_empty() {
                                    if state.selected_command >= app.settings.auto_execute_commands.len() {
                                        state.selected_command = app.settings.auto_execute_commands.len() - 1;
                                    }
                                } else {
                                    state.selected_command = 0;
                                }
                            }
                            Err(e) => {
                                app.error_message = Some(format!("Failed to save settings: {}", e));
                            }
                        }
                    }
                }
            }
            Message::SaveCommand => {
                if let Screen::Settings(ref mut state) = app.screen {
                    if let Some(ref textarea) = state.edit_textarea {
                        let command = textarea.lines().join("").trim().to_string();
                        if !command.is_empty() {
                            // Update in-memory settings
                            if let Some(idx) = state.edit_index {
                                // Editing existing command
                                if idx < app.settings.auto_execute_commands.len() {
                                    app.settings.auto_execute_commands[idx] = command;
                                }
                            } else {
                                // Adding new command
                                app.settings.auto_execute_commands.push(command);
                            }

                            // Save to disk
                            if let Err(e) = app.settings.save() {
                                app.error_message = Some(format!("Failed to save settings: {}", e));
                            }

                            state.mode = SettingsMode::List;
                            state.edit_textarea = None;
                            state.edit_index = None;
                        } else {
                            app.error_message = Some("Command cannot be empty".to_string());
                        }
                    }
                }
            }
            Message::CancelEdit => {
                if let Screen::Settings(ref mut state) = app.screen {
                    state.mode = SettingsMode::List;
                    state.edit_textarea = None;
                    state.edit_index = None;
                }
            }
            _ => {}
        }

        app.update(msg);
    }
}
