#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use benimator::AnimationPlugin;
use bevy::prelude::*;
use bevy::log;
use bevy::window::WindowMode;
use iyes_loopless::prelude::*;

mod assets;
mod camera;
mod debug;
mod game;
mod menu;
mod window;

const GAME_SIZE: (f32, f32) = (180.0, 320.0);
const DEFAULT_SCALE: u8 = 2;
const ALLOW_EXIT: bool = cfg!(not(target_arch = "wasm32"));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AppState {
    Loading,
    MainMenu,
    InGame,
}

fn main() {
    // When building for WASM, print panics to the browser console.
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    // TODO: Try to initialize logging before this. Maybe we can also make this code run in a plugin.
    let saved_window_state = window::load_window_state();

    let mut app = App::new();

    // Configure logging.
    if cfg!(feature = "verbose_logs") {
        let mut log_settings = log::LogSettings::default();
        log_settings.filter.push_str(",info,flappy_bevy=trace");
        log_settings.level = log::Level::TRACE;
        app.insert_resource(log_settings);
    } else if cfg!(debug_assertions) {
        let mut log_settings = log::LogSettings::default();
        log_settings.filter.push_str(",info,flappy_bevy=debug");
        log_settings.level = log::Level::DEBUG;
        app.insert_resource(log_settings);
    }

    app
        .insert_resource(WindowDescriptor {
            title: "Flappy Bevy".into(),
            width: GAME_SIZE.0 * saved_window_state.scale as f32,
            height: GAME_SIZE.1 * saved_window_state.scale as f32,
            resizable: false,
            position: saved_window_state.position.map(|pos| pos.as_vec2()),
            mode: WindowMode::Windowed,
            ..default()
        })
        .insert_resource(ClearColor(Color::rgb_u8(0, 57, 109)))

        // External plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_egui::EguiPlugin)
        .insert_resource(bevy_egui::EguiSettings {
            // TODO: Take DPI scaling into account as well.
            scale_factor: (saved_window_state.scale as f64) / (DEFAULT_SCALE as f64),
            ..default()
        })
        .add_plugin(bevy_kira_audio::AudioPlugin)
        .add_plugin(heron::PhysicsPlugin::default())
        .add_plugin(AnimationPlugin::default())

        // App setup
        .insert_resource(window::WindowScale(saved_window_state.scale))
        .add_loopless_state(AppState::Loading)
        .add_plugin(assets::AssetsPlugin)
        .add_plugin(debug::DebugPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(menu::MenuPlugin)
        .add_plugin(game::GamePlugin)
        .add_plugin(window::WindowPlugin);

    if ALLOW_EXIT {
        app.add_system(bevy::input::system::exit_on_esc_system);
    }

    app.run();
}
