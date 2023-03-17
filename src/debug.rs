use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_inspector_egui::{
    bevy_inspector,
    DefaultInspectorConfigPlugin,
};
use iyes_loopless::prelude::*;
use bevy_rapier2d::render::*;

const DEFAULT_SIZE: (f32, f32) = (320., 160.);

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        // NOTE: Copied code from bevy_inspector_egui::quick as suggested so we can customize the
        // inspector UI.
        if !app.is_plugin_added::<DefaultInspectorConfigPlugin>() {
            app.add_plugin(DefaultInspectorConfigPlugin);
        }
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugin(EguiPlugin);
        }

        app
            .add_plugin(RapierDebugRenderPlugin::default().disabled())
            .insert_resource(DebugUi::default())
            .add_system(debug_ui.run_if(debug_ui_enabled))
            .add_system(world_inspector_ui.run_if(show_world_inspector))
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
    mut egui_ctx: ResMut<EguiContext>,
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

// NOTE: Copied code from bevy_inspector_egui::quick as suggested so we can customize the
// inspector UI.
fn world_inspector_ui(world: &mut World) {
    let egui_context = world
        .resource_mut::<EguiContext>()
        .ctx_mut()
        .clone();

    egui::Window::new("World Inspector")
        .default_size(DEFAULT_SIZE)
        .show(&egui_context, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                bevy_inspector::ui_for_world(world, ui);
                ui.allocate_space(ui.available_size());
            });
        });
}

fn toggle_world_inspector(
    keys: ResMut<Input<KeyCode>>,
    mut debug_ui: ResMut<DebugUi>,
    mut egui_ctx: ResMut<EguiContext>,
) {
    if egui_ctx.ctx_mut().wants_keyboard_input() {
        return;
    }

    if keys.just_pressed(KeyCode::Back) {
        debug_ui.enabled = !debug_ui.enabled;
    }
}
