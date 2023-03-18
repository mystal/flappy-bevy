#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy::log::{self, LogPlugin};
use bevy::window::{WindowMode, WindowResolution};
use bevy_rapier2d::prelude::*;

mod animation;
mod assets;
mod camera;
mod debug;
mod game;
mod menu;
mod window;

const GAME_SIZE: (f32, f32) = (180.0, 320.0);
const DEFAULT_SCALE: u8 = 2;
const ALLOW_EXIT: bool = cfg!(not(target_arch = "wasm32"));

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum AppState {
    #[default]
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
    let log_plugin = {
        let mut plugin = LogPlugin::default();
        if cfg!(feature = "verbose_logs") {
            plugin.filter.push_str(",info,flappy_bevy=trace");
            plugin.level = log::Level::TRACE;
        } else if cfg!(debug_assertions) {
            plugin.filter.push_str(",info,flappy_bevy=debug");
            plugin.level = log::Level::DEBUG;
        }
        plugin
    };

    // Configure window.
    let window_position = saved_window_state.position
        .map(|pos| WindowPosition::At(pos))
        .unwrap_or(WindowPosition::Automatic);
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            title: "Flappy Bevy".into(),
            resolution: WindowResolution::new(
                GAME_SIZE.0 * saved_window_state.scale as f32,
                GAME_SIZE.1 * saved_window_state.scale as f32,
            ),
            resizable: false,
            position: window_position,
            mode: WindowMode::Windowed,
            ..default()
        }),
        ..default()
    };

    let default_plugins = DefaultPlugins
        .set(log_plugin)
        .set(ImagePlugin::default_nearest())
        .set(window_plugin);

    app
        .insert_resource(ClearColor(Color::rgb_linear(0.0, 57.0 / 255.0, 109.0 / 255.0)))

        // External plugins
        .add_plugins(default_plugins)
        .add_plugin(bevy_egui::EguiPlugin)
        .insert_resource(bevy_egui::EguiSettings {
            // TODO: Take DPI scaling into account as well.
            scale_factor: (saved_window_state.scale as f64) / (DEFAULT_SCALE as f64),
            ..default()
        })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.0))

        // App setup
        .insert_resource(window::WindowScale(saved_window_state.scale))
        .add_state::<AppState>()
        .add_plugin(assets::AssetsPlugin)
        .add_plugin(animation::AnimationPlugin)
        .add_plugin(debug::DebugPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(menu::MenuPlugin)
        .add_plugin(game::GamePlugin)
        .add_plugin(window::WindowPlugin);

    if ALLOW_EXIT {
        app.add_system(bevy::window::close_on_esc);
    }

    app.run();
}
