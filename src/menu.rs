use bevy::prelude::*;
use bevy::app::AppExit;
use bevy_egui::{egui, EguiContexts};

use crate::{
    ALLOW_EXIT, AppState,
    assets::GameAssets,
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(setup_main_menu.in_schedule(OnEnter(AppState::MainMenu)))
            .add_system(despawn_main_menu.in_schedule(OnExit(AppState::MainMenu)))
            // TODO: Temp hack to work around bevy_egui not supporting touches. Remove once it does!
            .add_system(main_menu_ui.in_set(OnUpdate(AppState::MainMenu)));

        if cfg!(target_arch = "wasm32") {
            app.add_system(tap_to_start.in_set(OnUpdate(AppState::MainMenu)));
        }
    }
}

fn setup_main_menu(
    mut commands: Commands,
    assets: Res<GameAssets>,
) {
    // 2D camera to view Title Text
    commands.spawn(Camera2dBundle::default());

    let style = TextStyle {
        font: assets.font.clone(),
        font_size: 80.0,
        color: Color::WHITE,
    };
    let alignment = TextAlignment::Center;
    commands
        .spawn(Text2dBundle {
            text: Text::from_section("Flappy\nBevy", style.clone())
                .with_alignment(alignment),
            transform: Transform::from_translation(Vec3::new(0.0, 200.0, 0.0)),
            ..default()
        });
}

fn despawn_main_menu(
    mut commands: Commands,
    query: Query<Entity, Or<(With<Camera>, With<Text>)>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn tap_to_start(
    buttons: Res<Input<MouseButton>>,
    touches: Res<Touches>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let mouse_input = buttons.just_pressed(MouseButton::Left);
    // TODO: This doesn't work cause bevy/winit don't support web touch events.
    let touch_input = touches.iter_just_pressed().count() > 0;
    if mouse_input || touch_input {
        next_state.set(AppState::InGame);
    }
}

fn main_menu_ui(
    mut next_state: ResMut<NextState<AppState>>,
    mut ctx: EguiContexts,
    mut exit: EventWriter<AppExit>,
) {
    let window = egui::Window::new("Main Menu")
        .title_bar(false)
        .auto_sized()
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -50.0])
        .frame(egui::Frame::none());
    window.show(ctx.ctx_mut(), |ui| {
        ui.set_width(250.0);
        ui.vertical_centered_justified(|ui| {
            let play = egui::RichText::new("Play").size(60.0);
            if ui.button(play).clicked() {
                next_state.set(AppState::InGame);
            }

            if ALLOW_EXIT {
                let quit = egui::RichText::new("Quit").size(60.0);
                if ui.button(quit).clicked() {
                    exit.send(AppExit);
                }
            }
        });
    });
}
