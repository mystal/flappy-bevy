#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use iyes_loopless::prelude::*;
use serde::{Deserialize, Serialize};

mod assets;
mod debug;
mod game;
mod menu;

const GAME_SIZE: (f32, f32) = (180.0, 320.0);
const DEFAULT_SCALE: u8 = 2;
const WINDOW_SIZE: (f32, f32) = (GAME_SIZE.0 * DEFAULT_SCALE as f32, GAME_SIZE.1 * DEFAULT_SCALE as f32);
const ALLOW_EXIT: bool = cfg!(not(target_arch = "wasm32"));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AppState {
    MainMenu,
    InGame,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct SavedWindowState {
    position: Option<Vec2>,
}

fn main() {
    // When building for WASM, print panics to the browser console.
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    // Try to load window state.
    let window_state_filename = "window_state.toml";
    let saved_window_state: SavedWindowState = if std::path::Path::new(window_state_filename).is_file() {
        let window_toml_str = std::fs::read_to_string(window_state_filename).unwrap();
        toml::from_str(&window_toml_str).unwrap()
    } else {
        default()
    };

    let mut app = App::new();
    app
        .insert_resource(WindowDescriptor {
            title: "Flappy Bevy".into(),
            width: WINDOW_SIZE.0,
            height: WINDOW_SIZE.1,
            resizable: false,
            position: saved_window_state.position,
            ..default()
        })
        // .insert_resource(ClearColor(Color::hex("018893").unwrap()))
        .insert_resource(ClearColor(Color::rgb_u8(230, 230, 230)))

        // External plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_egui::EguiPlugin)
        .add_plugin(bevy_kira_audio::AudioPlugin)
        .add_plugin(heron::PhysicsPlugin::default())

        // App setup
        .add_loopless_state(AppState::MainMenu)
        .add_plugin(assets::AssetsPlugin)
        .add_plugin(debug::DebugPlugin)
        .add_plugin(menu::MenuPlugin)
        .add_plugin(game::GamePlugin);

    if ALLOW_EXIT {
        app.add_system(bevy::input::system::exit_on_esc_system);
    }

    app.run();
}
