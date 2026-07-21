use crate::app::{App, Screen};
use crate::ui::screens;
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &mut App) {
    match &mut app.screen {
        Screen::CredentialInput(state) => {
            screens::credential_input::render(f, state, app.error_message.as_deref());
        }
        Screen::InstanceList => {
            // Create an immutable reference for reading
            let region = &app.region;
            let instances = &app.instances;
            let selected_instance = app.selected_instance;
            let error_message = &app.error_message;
            let info_message = &app.info_message;
            let settings = &app.settings;
            screens::instance_list::render_with_data(f, region, instances, selected_instance, error_message, info_message, settings);
        }
        Screen::RegionSelection { selected } => {
            let region = &app.region;
            screens::region_selection::render_with_data(f, region, *selected);
        }
        Screen::Help => {
            screens::help::render(f);
        }
        Screen::Settings(state) => {
            screens::settings::render(f, state, &app.settings);
        }
        Screen::PortForwards(state) => {
            screens::port_forwards::render(f, state, &app.settings);
        }
    }

    // Render loading overlay if loading
    if !matches!(app.loading, crate::app::LoadingState::Idle) {
        screens::loading::render_overlay(f, &app.loading);
    }
}
