use std::fs;
use std::path::Path;

use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::window::PrimaryWindow;
use serde::{Deserialize, Serialize};

use crate::DEFAULT_SCALE;

const WINDOW_STATE_FILENAME: &str = "window_state.ron";

#[derive(Clone, Debug, Deserialize, Serialize, Resource)]
pub struct WindowState {
    #[serde(default)]
    pub position: WindowPosition,
    #[serde(default)]
    pub scale: u8,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            position: WindowPosition::Automatic,
            scale: DEFAULT_SCALE,
        }
    }
}

pub fn load_window_state() -> WindowState {
    if Path::new(WINDOW_STATE_FILENAME).is_file() {
        // TODO: Log errors if these fail and return default.
        let window_state_str = fs::read_to_string(WINDOW_STATE_FILENAME)
            .expect("Could not read window state file");
        ron::from_str(&window_state_str)
            .expect("Could not deserialize window state")
    } else {
        default()
    }
}

pub struct WindowPlugin {
    saved_window_state: WindowState,
}

impl WindowPlugin {
    pub fn new(saved_window_state: WindowState) -> Self {
        Self {
            saved_window_state,
        }
    }
}

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(self.saved_window_state.clone())
            .add_system(update_window_state.in_base_set(CoreSet::PostUpdate));
        #[cfg(not(target_arch = "wasm32"))]
        app.add_system(
            save_window_state_on_exit
                .in_base_set(CoreSet::Last)
                .run_if(on_event::<AppExit>())
        );
    }
}

fn update_window_state(
    mut window_state: ResMut<WindowState>,
    window_q: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
) {
    if let Ok(window) = window_q.get_single() {
        window_state.position = window.position;
    }
}

fn save_window_state_on_exit(
    window_state: Res<WindowState>,
) {
    info!("Saving window state");

    let pretty_config = ron::ser::PrettyConfig::default();
    let state_str = ron::ser::to_string_pretty(&*window_state, pretty_config)
        .expect("Could not serialize window state");
    fs::write(WINDOW_STATE_FILENAME, state_str)
        .expect("Could not write window state to file");
}
