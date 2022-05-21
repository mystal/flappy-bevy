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
    _server: Res<AssetServer>,
    mut _assets: ResMut<Assets>,
) {
}
