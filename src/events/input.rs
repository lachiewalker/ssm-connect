use crate::app::{App, Message, PortForwardsMode, Screen, SettingsMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key_event(app: &App, key: KeyEvent) -> Option<Message> {
    match &app.screen {
        Screen::CredentialInput(_state) => {
            match key.code {
                KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Some(Message::Quit)
                }
                KeyCode::Esc => Some(Message::ExitCredentialInput),
                KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                    Some(Message::PreviousField)
                }
                KeyCode::Tab => Some(Message::NextField),
                KeyCode::Enter => Some(Message::SubmitCredentials),
                KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Some(Message::UseDefaultCredentials)
                }
                _ => None, // Let TextArea handle other keys
            }
        }
        Screen::InstanceList => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(Message::Quit),
            KeyCode::Up | KeyCode::Char('k') => Some(Message::NavigateUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Message::NavigateDown),
            KeyCode::Enter => Some(Message::ConnectToInstance),
            KeyCode::Char('s') => Some(Message::StartInstance),
            KeyCode::Char('r') => Some(Message::ChangeRegion),
            KeyCode::Char('c') => Some(Message::OpenSettings),
            KeyCode::Char('p') => Some(Message::OpenPortForwards),
            KeyCode::Char('?') => Some(Message::ToggleHelp),
            _ => {
                if app.error_message.is_some() || app.info_message.is_some() {
                    Some(Message::ClearError)
                } else {
                    None
                }
            }
        },
        Screen::RegionSelection { .. } => match key.code {
            KeyCode::Char('q') => Some(Message::Quit),
            KeyCode::Esc => Some(Message::Back),
            KeyCode::Up | KeyCode::Char('k') => Some(Message::NavigateUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Message::NavigateDown),
            KeyCode::Enter => Some(Message::Select),
            _ => None,
        },
        Screen::Help => match key.code {
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => Some(Message::ToggleHelp),
            _ => None,
        },
        Screen::Settings(state) => {
            match state.mode {
                SettingsMode::List => match key.code {
                    KeyCode::Esc => Some(Message::Back),
                    KeyCode::Up | KeyCode::Char('k') => Some(Message::NavigateUp),
                    KeyCode::Down | KeyCode::Char('j') => Some(Message::NavigateDown),
                    KeyCode::Char('a') => Some(Message::AddCommand),
                    KeyCode::Char('e') => Some(Message::EditCommand),
                    KeyCode::Char('d') => Some(Message::DeleteCommand),
                    _ => None,
                },
                SettingsMode::Edit => match key.code {
                    KeyCode::Enter => Some(Message::SaveCommand),
                    KeyCode::Esc => Some(Message::CancelEdit),
                    _ => None,  // TextArea handles other keys
                },
            }
        },
        Screen::PortForwards(state) => {
            match state.mode {
                PortForwardsMode::InstanceToggle => match key.code {
                    KeyCode::Esc => Some(Message::Back),
                    KeyCode::Up | KeyCode::Char('k') => Some(Message::NavigateUp),
                    KeyCode::Down | KeyCode::Char('j') => Some(Message::NavigateDown),
                    KeyCode::Char(' ') | KeyCode::Enter => Some(Message::TogglePortForward),
                    KeyCode::Char('r') => Some(Message::OpenPortForwardRules),
                    _ => None,
                },
                PortForwardsMode::GlobalEdit => match key.code {
                    KeyCode::Esc => Some(Message::Back),
                    KeyCode::Up | KeyCode::Char('k') => Some(Message::NavigateUp),
                    KeyCode::Down | KeyCode::Char('j') => Some(Message::NavigateDown),
                    KeyCode::Char('a') => Some(Message::AddPortForwardRule),
                    KeyCode::Char('e') => Some(Message::EditPortForwardRule),
                    KeyCode::Char('d') => Some(Message::DeletePortForwardRule),
                    _ => None,
                },
                PortForwardsMode::EditRule => match key.code {
                    KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        Some(Message::PreviousPortForwardField)
                    }
                    KeyCode::Tab => Some(Message::NextPortForwardField),
                    KeyCode::Enter => Some(Message::SavePortForwardRule),
                    KeyCode::Esc => Some(Message::CancelPortForwardEdit),
                    _ => None, // TextArea handles other keys
                },
            }
        },
    }
}
