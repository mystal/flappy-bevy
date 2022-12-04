use std::fs;
use std::path::Path;

use bevy::prelude::*;
use bevy::app::AppExit;
use serde::{Deserialize, Serialize};

use crate::DEFAULT_SCALE;

const WINDOW_STATE_FILENAME: &str = "window_state.toml";

#[derive(Debug, Deserialize, Serialize)]
pub struct SavedWindowState {
    pub position: Option<IVec2>,
    #[serde(default)]
    pub scale: u8,
}

impl Default for SavedWindowState {
    fn default() -> Self {
        Self {
            position: None,
            scale: DEFAULT_SCALE,
        }
    }
}

#[derive(Resource)]
pub struct WindowScale(pub u8);

pub fn load_window_state() -> SavedWindowState {
    if Path::new(WINDOW_STATE_FILENAME).is_file() {
        // TODO: Log errors if these fail and return default.
        let window_toml_str = fs::read_to_string(WINDOW_STATE_FILENAME).unwrap();
        toml::from_str(&window_toml_str).unwrap()
    } else {
        default()
    }
}

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_arch = "wasm32"))]
        app.add_system_to_stage(CoreStage::PostUpdate, save_window_state_on_exit);
    }
}

fn save_window_state_on_exit(
    mut exit_events: EventReader<AppExit>,
    windows: Res<Windows>,
    window_scale: Res<WindowScale>,
) {
    // Call last to iterate over all the exit events.
    if exit_events.iter().last().is_none() {
        // If the last element is None, it means we don't have any events, so not exiting yet.
        return;
    }

    if let Some(window) = windows.get_primary() {
        info!("Saving window state");

        if let Some(position) = window.position() {
            let window_state = SavedWindowState {
                position: Some(position),
                scale: window_scale.0,
            };
            let state_str = toml::to_string(&window_state).unwrap();
            fs::write(WINDOW_STATE_FILENAME, state_str).unwrap();
        }
    }
}
