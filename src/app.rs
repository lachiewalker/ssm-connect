use crate::aws::types::{AwsCredentials, EC2Instance};
use crate::config::Settings;
use tui_textarea::TextArea;

#[derive(Debug)]
pub struct App {
    pub screen: Screen,
    pub credentials: Option<AwsCredentials>,
    pub region: String,
    pub instances: Vec<EC2Instance>,
    pub selected_instance: Option<usize>,
    pub should_quit: bool,
    pub loading: LoadingState,
    pub error_message: Option<String>,
    pub info_message: Option<String>,
    pub pending_connection: Option<String>, // Instance ID to connect to
    pub settings: Settings, // Settings cached in memory
    pub polling_instance_state: bool, // Track if we should poll for instance state changes
}

#[derive(Debug)]
pub enum Screen {
    CredentialInput(CredentialInputState),
    InstanceList,
    RegionSelection { selected: usize },
    Help,
    Settings(SettingsScreenState),
    PortForwards(PortForwardsScreenState),
}

#[derive(Debug)]
pub struct CredentialInputState {
    pub access_key: TextArea<'static>,
    pub secret_key: TextArea<'static>,
    pub session_token: TextArea<'static>,
    pub focused_field: CredentialField,
    pub hint_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CredentialField {
    AccessKey,
    SecretKey,
    SessionToken,
}

#[derive(Debug)]
pub struct SettingsScreenState {
    pub mode: SettingsMode,
    pub selected_command: usize,
    pub edit_textarea: Option<TextArea<'static>>,
    pub edit_index: Option<usize>,  // None = adding, Some(i) = editing
}

#[derive(Debug, PartialEq)]
pub enum SettingsMode {
    List,   // Viewing command list
    Edit,   // Adding/editing a command
}

#[derive(Debug)]
pub struct PortForwardsScreenState {
    pub mode: PortForwardsMode,
    pub instance_id: String,
    pub instance_name: String,
    pub selected_rule: usize,
    pub edit_state: Option<PortForwardEditState>,
}

#[derive(Debug, PartialEq)]
pub enum PortForwardsMode {
    InstanceToggle,
    GlobalEdit,
    EditRule,
}

#[derive(Debug)]
pub struct PortForwardEditState {
    pub alias: TextArea<'static>,
    pub local_port: TextArea<'static>,
    pub remote_port: TextArea<'static>,
    pub focused_field: PortForwardField,
    pub edit_index: Option<usize>, // None = new, Some(i) = editing
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortForwardField {
    Alias,
    LocalPort,
    RemotePort,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadingState {
    Idle,
    ValidatingCredentials,
    LoadingInstances,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Input messages
    Quit,
    NavigateUp,
    NavigateDown,
    Select,
    Back,
    ChangeRegion,
    ToggleHelp,
    ClearError,

    // Credential messages
    SubmitCredentials,
    UseDefaultCredentials,
    ExitCredentialInput,
    CredentialsValidated(AwsCredentials, String), // credentials, region
    CredentialValidationFailed(String),
    NextField,
    PreviousField,

    // AWS operation messages
    InstancesLoaded(Vec<EC2Instance>),
    ReloadInstances,
    StartInstance,
    InstanceStarted(String), // instance_id
    ConnectToInstance,
    RegionChanged(String),

    // Settings messages
    OpenSettings,
    AddCommand,
    EditCommand,
    DeleteCommand,
    SaveCommand,
    CancelEdit,

    // Port forwarding messages
    OpenPortForwards,
    TogglePortForward,
    OpenPortForwardRules,
    AddPortForwardRule,
    EditPortForwardRule,
    DeletePortForwardRule,
    SavePortForwardRule,
    CancelPortForwardEdit,
    NextPortForwardField,
    PreviousPortForwardField,

    // Error messages
    Error(String),
}

impl App {
    pub fn new(region: String, settings: Settings) -> Self {
        Self {
            screen: Screen::CredentialInput(CredentialInputState::new()),
            credentials: None,
            region,
            instances: Vec::new(),
            selected_instance: None,
            should_quit: false,
            loading: LoadingState::Idle,
            error_message: None,
            info_message: None,
            pending_connection: None,
            settings,
            polling_instance_state: false,
        }
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::Quit => {
                self.should_quit = true;
            }
            Message::NavigateUp => {
                if let Screen::InstanceList = self.screen {
                    if !self.instances.is_empty() {
                        self.selected_instance = Some(match self.selected_instance {
                            Some(idx) if idx > 0 => idx - 1,
                            Some(_) => self.instances.len() - 1,
                            None => self.instances.len() - 1,
                        });
                    }
                } else if let Screen::RegionSelection { ref mut selected } = self.screen {
                    if *selected > 0 {
                        *selected -= 1;
                    }
                } else if let Screen::Settings(ref mut state) = self.screen {
                    if state.mode == SettingsMode::List {
                        if !self.settings.auto_execute_commands.is_empty() {
                            if state.selected_command > 0 {
                                state.selected_command -= 1;
                            } else {
                                state.selected_command = self.settings.auto_execute_commands.len() - 1;
                            }
                        }
                    }
                } else if let Screen::PortForwards(ref mut state) = self.screen {
                    match state.mode {
                        PortForwardsMode::InstanceToggle => {
                            let len = self.settings.port_forwarding_rules.len();
                            if len > 0 {
                                if state.selected_rule > 0 {
                                    state.selected_rule -= 1;
                                } else {
                                    state.selected_rule = len - 1;
                                }
                            }
                        }
                        PortForwardsMode::GlobalEdit => {
                            let len = self.settings.port_forwarding_rules.len();
                            if len > 0 {
                                if state.selected_rule > 0 {
                                    state.selected_rule -= 1;
                                } else {
                                    state.selected_rule = len - 1;
                                }
                            }
                        }
                        PortForwardsMode::EditRule => {}
                    }
                }
            }
            Message::NavigateDown => {
                if let Screen::InstanceList = self.screen {
                    if !self.instances.is_empty() {
                        self.selected_instance = Some(match self.selected_instance {
                            Some(idx) if idx < self.instances.len() - 1 => idx + 1,
                            Some(_) => 0,
                            None => 0,
                        });
                    }
                } else if let Screen::RegionSelection { ref mut selected } = self.screen {
                    let regions = get_aws_regions();
                    if *selected < regions.len() - 1 {
                        *selected += 1;
                    }
                } else if let Screen::Settings(ref mut state) = self.screen {
                    if state.mode == SettingsMode::List {
                        if !self.settings.auto_execute_commands.is_empty() {
                            if state.selected_command < self.settings.auto_execute_commands.len() - 1 {
                                state.selected_command += 1;
                            } else {
                                state.selected_command = 0;
                            }
                        }
                    }
                } else if let Screen::PortForwards(ref mut state) = self.screen {
                    match state.mode {
                        PortForwardsMode::InstanceToggle => {
                            let len = self.settings.port_forwarding_rules.len();
                            if len > 0 {
                                if state.selected_rule < len - 1 {
                                    state.selected_rule += 1;
                                } else {
                                    state.selected_rule = 0;
                                }
                            }
                        }
                        PortForwardsMode::GlobalEdit => {
                            let len = self.settings.port_forwarding_rules.len();
                            if len > 0 {
                                if state.selected_rule < len - 1 {
                                    state.selected_rule += 1;
                                } else {
                                    state.selected_rule = 0;
                                }
                            }
                        }
                        PortForwardsMode::EditRule => {}
                    }
                }
            }
            Message::Select => {
                // Region selection handled in event loop via RegionChanged message
            }
            Message::Back => {
                if let Screen::Help = self.screen {
                    self.screen = Screen::InstanceList;
                } else if let Screen::RegionSelection { .. } = self.screen {
                    self.screen = Screen::InstanceList;
                } else if let Screen::Settings(ref state) = self.screen {
                    if state.mode == SettingsMode::Edit {
                        // Handled by CancelEdit message
                    } else {
                        self.screen = Screen::InstanceList;
                    }
                } else if let Screen::PortForwards(ref mut state) = self.screen {
                    match state.mode {
                        PortForwardsMode::InstanceToggle => {
                            self.screen = Screen::InstanceList;
                        }
                        PortForwardsMode::GlobalEdit => {
                            state.mode = PortForwardsMode::InstanceToggle;
                            state.selected_rule = 0;
                        }
                        PortForwardsMode::EditRule => {
                            // Handled by CancelPortForwardEdit
                        }
                    }
                }
            }
            Message::ChangeRegion => {
                if let Screen::InstanceList = self.screen {
                    let regions = get_aws_regions();
                    let current_idx = regions.iter().position(|r| r == &self.region).unwrap_or(0);
                    self.screen = Screen::RegionSelection { selected: current_idx };
                }
            }
            Message::ToggleHelp => {
                match self.screen {
                    Screen::Help => {
                        self.screen = Screen::InstanceList;
                    }
                    Screen::InstanceList => {
                        self.screen = Screen::Help;
                    }
                    _ => {}
                }
            }
            Message::ExitCredentialInput => {
                // If credentials already exist, user is editing - go back to instance list
                // Otherwise, quit since we can't proceed without credentials
                if self.credentials.is_some() {
                    self.screen = Screen::InstanceList;
                } else {
                    self.should_quit = true;
                }
            }
            Message::CredentialsValidated(_creds, _region) => {
                // Handled in event handler
            }
            Message::NextField => {
                if let Screen::CredentialInput(ref mut state) = self.screen {
                    state.next_field();
                }
            }
            Message::PreviousField => {
                if let Screen::CredentialInput(ref mut state) = self.screen {
                    state.previous_field();
                }
            }
            Message::NextPortForwardField => {
                if let Screen::PortForwards(ref mut state) = self.screen {
                    if let Some(ref mut edit) = state.edit_state {
                        edit.focused_field = match edit.focused_field {
                            PortForwardField::Alias => PortForwardField::LocalPort,
                            PortForwardField::LocalPort => PortForwardField::RemotePort,
                            PortForwardField::RemotePort => PortForwardField::Alias,
                        };
                    }
                }
            }
            Message::PreviousPortForwardField => {
                if let Screen::PortForwards(ref mut state) = self.screen {
                    if let Some(ref mut edit) = state.edit_state {
                        edit.focused_field = match edit.focused_field {
                            PortForwardField::Alias => PortForwardField::RemotePort,
                            PortForwardField::LocalPort => PortForwardField::Alias,
                            PortForwardField::RemotePort => PortForwardField::LocalPort,
                        };
                    }
                }
            }
            Message::CredentialValidationFailed(err) => {
                self.error_message = Some(format!("Credential validation failed: {}", err));
                self.loading = LoadingState::Idle;
            }
            Message::InstancesLoaded(instances) => {
                self.instances = instances;
                self.loading = LoadingState::Idle;
                if !self.instances.is_empty() && self.selected_instance.is_none() {
                    self.selected_instance = Some(0);
                }

                // Check if any instances are in transitional states
                use crate::aws::types::InstanceState;
                let has_transitional = self.instances.iter().any(|i|
                    matches!(i.state, InstanceState::Pending | InstanceState::Stopping)
                );

                // Stop polling if no instances are in transitional states
                if !has_transitional {
                    self.polling_instance_state = false;
                    self.info_message = None;
                }
            }
            Message::InstanceStarted(instance_id) => {
                self.loading = LoadingState::Idle;
                self.info_message = Some(format!("Instance {} is starting", instance_id));
                self.polling_instance_state = true; // Enable polling until instance is running
                // Will reload instances to show new state
            }
            Message::Error(err) => {
                self.error_message = Some(err);
                self.loading = LoadingState::Idle;
            }
            Message::ClearError => {
                self.error_message = None;
                self.info_message = None;
            }
            Message::RegionChanged(new_region) => {
                self.region = new_region;
                self.instances.clear();
                self.selected_instance = None;
                self.loading = LoadingState::LoadingInstances;
            }
            _ => {}
        }
    }

    pub fn selected_instance(&self) -> Option<&EC2Instance> {
        self.selected_instance.and_then(|idx| self.instances.get(idx))
    }
}

impl CredentialInputState {
    pub fn new() -> Self {
        let mut access_key = TextArea::default();
        access_key.set_placeholder_text("AWS_ACCESS_KEY_ID");

        let mut secret_key = TextArea::default();
        secret_key.set_placeholder_text("AWS_SECRET_ACCESS_KEY");
        secret_key.set_mask_char('*');

        let mut session_token = TextArea::default();
        session_token.set_placeholder_text("AWS_SESSION_TOKEN (optional)");

        Self {
            access_key,
            secret_key,
            session_token,
            focused_field: CredentialField::AccessKey,
            hint_message: None,
        }
    }

    pub fn next_field(&mut self) {
        self.focused_field = match self.focused_field {
            CredentialField::AccessKey => CredentialField::SecretKey,
            CredentialField::SecretKey => CredentialField::SessionToken,
            CredentialField::SessionToken => CredentialField::AccessKey,
        };
    }

    pub fn previous_field(&mut self) {
        self.focused_field = match self.focused_field {
            CredentialField::AccessKey => CredentialField::SessionToken,
            CredentialField::SecretKey => CredentialField::AccessKey,
            CredentialField::SessionToken => CredentialField::SecretKey,
        };
    }

    pub fn current_field_mut(&mut self) -> &mut TextArea<'static> {
        match self.focused_field {
            CredentialField::AccessKey => &mut self.access_key,
            CredentialField::SecretKey => &mut self.secret_key,
            CredentialField::SessionToken => &mut self.session_token,
        }
    }

    pub fn get_credentials(&mut self) -> Option<AwsCredentials> {
        let access_key_text = self.access_key.lines().join("").trim().to_string();
        let secret_key_text = self.secret_key.lines().join("").trim().to_string();
        let session_token_text = self.session_token.lines().join("").trim().to_string();

        // Check if both required fields are filled
        if !access_key_text.is_empty() && !secret_key_text.is_empty() {
            self.hint_message = None;
            return Some(AwsCredentials {
                access_key_id: access_key_text,
                secret_access_key: secret_key_text,
                session_token: if session_token_text.is_empty() {
                    None
                } else {
                    Some(session_token_text)
                },
            });
        }

        // Only access key filled - show hint
        if !access_key_text.is_empty() && secret_key_text.is_empty() {
            self.hint_message = Some(
                "Tip: Paste the entire AWS credential export block (all lines), or fill in the secret key field"
                    .to_string()
            );
            return None;
        }

        // Nothing filled
        self.hint_message = None;
        None
    }
}

pub fn get_aws_regions() -> Vec<&'static str> {
    vec![
        "us-east-1",
        "us-east-2",
        "us-west-1",
        "us-west-2",
        "af-south-1",
        "ap-east-1",
        "ap-south-1",
        "ap-northeast-1",
        "ap-northeast-2",
        "ap-northeast-3",
        "ap-southeast-1",
        "ap-southeast-2",
        "ca-central-1",
        "eu-central-1",
        "eu-west-1",
        "eu-west-2",
        "eu-west-3",
        "eu-north-1",
        "eu-south-1",
        "me-south-1",
        "sa-east-1",
    ]
}
