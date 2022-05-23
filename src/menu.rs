use bevy::prelude::*;
use bevy::app::AppExit;
use bevy_egui::{egui, EguiContext};
use iyes_loopless::prelude::*;

use crate::{
    ALLOW_EXIT, AppState,
    assets::Assets,
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_enter_system(AppState::MainMenu, setup_main_menu)
            .add_exit_system(AppState::MainMenu, despawn_main_menu)
            // TODO: Temp hack to work around bevy_egui not supporting touches. Remove once it does!
            .add_system(tap_to_start.run_in_state(AppState::MainMenu))
            .add_system(main_menu_ui.run_in_state(AppState::MainMenu));
    }
}

fn setup_main_menu(
    mut commands: Commands,
    assets: Res<Assets>,
) {
    // 2D camera to view Title Text
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let style = TextStyle {
        font: assets.font.clone(),
        font_size: 80.0,
        color: Color::BLACK,
    };
    let alignment = TextAlignment {
        horizontal: HorizontalAlign::Center,
        ..default()
    };
    commands
        .spawn_bundle(Text2dBundle {
            text: Text::with_section("Flappy\nBevy", style.clone(), alignment),
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
    touches: Res<Touches>,
    mut commands: Commands,
) {
    // TODO: This isn't working and I don't know why!
    let touch_input = touches.iter_just_pressed().count() > 0;
    if touch_input {
        commands.insert_resource(NextState(AppState::InGame));
    }
}

fn main_menu_ui(
    mut commands: Commands,
    mut ctx: ResMut<EguiContext>,
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
                commands.insert_resource(NextState(AppState::InGame));
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
