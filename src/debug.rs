use bevy::prelude::*;
use bevy_egui::EguiContext;
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(WorldInspectorParams {
                enabled: false,
                ..default()
            })
            .add_plugin(WorldInspectorPlugin::new())
            .add_system(toggle_world_inspector);
    }
}

fn toggle_world_inspector(
    keys: ResMut<Input<KeyCode>>,
    mut inspector_params: ResMut<WorldInspectorParams>,
    mut egui_ctx: ResMut<EguiContext>,
) {
    if egui_ctx.ctx_mut().wants_keyboard_input() {
        return;
    }

    if keys.just_pressed(KeyCode::Back) {
        inspector_params.enabled = !inspector_params.enabled;
    }
}
