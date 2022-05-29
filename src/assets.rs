use std::time::Duration;

use benimator::SpriteSheetAnimation;
use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};
use iyes_loopless::prelude::*;

use crate::AppState;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        AssetLoader::new(LoadingState::Loading)
            .continue_to_state(LoadingState::Done)
            .with_collection::<GameAssets>()
            .build(app);
        app.add_state(LoadingState::Loading)
            .add_system_set(SystemSet::on_enter(LoadingState::Done).with_system(assets_loaded));
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum LoadingState {
    Loading,
    Done,
}

#[derive(AssetCollection)]
pub struct GameAssets {
    #[asset(path = "fonts/Kenney Blocks.ttf")]
    pub font: Handle<Font>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 4, rows = 1))]
    #[asset(path = "sprites/Player/bird1.png")]
    pub bird_atlas: Handle<TextureAtlas>,
    pub bird_anim: Handle<SpriteSheetAnimation>,

    #[asset(path = "sprites/Background/Background5.png")]
    pub background: Handle<Image>,
}

fn assets_loaded(
    mut commands: Commands,
    mut assets: ResMut<GameAssets>,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
) {
    eprintln!("Loaded assets!");

    // Bird anim info asset.
    let bird_anim = SpriteSheetAnimation::from_range(0..=3, Duration::from_millis(150));
    assets.bird_anim = animations.add(bird_anim);

    // Go to main menu!
    commands.insert_resource(NextState(AppState::MainMenu));
}
