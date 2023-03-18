use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::render::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(WorldInspectorPlugin::default().run_if(show_world_inspector))
            .add_plugin(RapierDebugRenderPlugin::default().disabled())
            .insert_resource(DebugUi::default())
            .add_system(debug_ui.run_if(debug_ui_enabled))
            .add_system(toggle_world_inspector);
    }
}

#[derive(Default, Resource)]
struct DebugUi {
    enabled: bool,
    show_world_inspector: bool,
}

fn debug_ui_enabled(
    debug_ui: Res<DebugUi>,
) -> bool {
    debug_ui.enabled
}

fn show_world_inspector(
    debug_ui: Res<DebugUi>,
) -> bool {
    debug_ui.enabled && debug_ui.show_world_inspector
}

fn debug_ui(
    mut debug_ui: ResMut<DebugUi>,
    mut debug_physics_ctx: ResMut<DebugRenderContext>,
    mut egui_ctx: EguiContexts,
) {
    let ctx = egui_ctx.ctx_mut();

    egui::TopBottomPanel::top("debug_panel")
        .show(ctx, |ui| {
            // NOTE: An egui bug makes clicking on the menu bar not report wants_pointer_input,
            // which means it'll register as a click in game.
            // https://github.com/emilk/egui/issues/2606
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Debug", |ui| {
                    ui.checkbox(&mut debug_ui.show_world_inspector, "World Inspector");
                    ui.checkbox(&mut debug_physics_ctx.enabled, "Debug Physics Render");
                });
            });
        });
}

fn toggle_world_inspector(
    keys: ResMut<Input<KeyCode>>,
    mut debug_ui: ResMut<DebugUi>,
    mut egui_ctx: EguiContexts,
) {
    if egui_ctx.ctx_mut().wants_keyboard_input() {
        return;
    }

    if keys.just_pressed(KeyCode::Back) {
        debug_ui.enabled = !debug_ui.enabled;
    }
}
