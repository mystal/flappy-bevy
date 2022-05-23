use bevy::prelude::*;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<Assets>()
            .add_startup_system(load_assets);
    }
}

#[derive(Default)]
pub struct Assets {
    pub font: Handle<Font>,
}

pub fn load_assets(
    server: Res<AssetServer>,
    mut assets: ResMut<Assets>,
) {
    assets.font = server.load("fonts/Kenney Blocks.ttf");
}
