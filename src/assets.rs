use std::time::Duration;

use benimator::SpriteSheetAnimation;
use bevy::prelude::*;
use bevy::{
    sprite::Rect,
    render::render_resource::AddressMode,
};
use bevy_asset_loader::{AssetCollection, AssetLoader};
use iyes_loopless::prelude::*;

use crate::AppState;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        AssetLoader::new(AppState::Loading)
            .continue_to_state(AppState::MainMenu)
            .with_collection::<GameAssets>()
            .build(app);
        app.add_exit_system(AppState::Loading, assets_loaded);
    }
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

    #[asset(path = "sprites/Tileset/Style 1/OldS2.png")]
    pub terrain_image: Handle<Image>,
    pub terrain_atlas: Handle<TextureAtlas>,
    pub terrain_indices: TerrainAtlasIndices,

    #[asset(path = "sprites/Tileset/Style 1/OldS2-PipeCenter.png")]
    pub pipe_center: Handle<Image>,
    #[asset(path = "sprites/Tileset/Style 1/OldS2-GroundTop.png")]
    pub ground_top: Handle<Image>,
    #[asset(path = "sprites/Tileset/Style 1/OldS2-Ground.png")]
    pub ground: Handle<Image>,
}

#[derive(Default)]
pub struct TerrainAtlasIndices {
    pub pipe_top: usize,
    pub pipe_bottom: usize,
    pub pipe_center: usize,

    pub ground_top: usize,
    pub ground: usize,
}

fn assets_loaded(
    mut assets: ResMut<GameAssets>,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut images: ResMut<Assets<Image>>,
) {
    eprintln!("Loaded assets!");

    // Bird anim info asset.
    let bird_anim = SpriteSheetAnimation::from_range(0..=3, Duration::from_millis(150));
    assets.bird_anim = animations.add(bird_anim);

    // Populate terrain texture atlas.
    if let Some(image) = images.get_mut(&assets.terrain_image) {
        image.sampler_descriptor.address_mode_u = AddressMode::Repeat;
        image.sampler_descriptor.address_mode_v = AddressMode::Repeat;

        let mut atlas = TextureAtlas::new_empty(assets.terrain_image.clone(), image.size());
        assets.terrain_indices.pipe_top = atlas.add_texture(Rect {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(32.0, 16.0),
        });
        assets.terrain_indices.pipe_bottom = atlas.add_texture(Rect {
            min: Vec2::new(0.0, 64.0),
            max: Vec2::new(32.0, 80.0),
        });
        assets.terrain_indices.pipe_center = atlas.add_texture(Rect {
            min: Vec2::new(2.0, 32.0),
            max: Vec2::new(30.0, 48.0),
        });
        assets.terrain_indices.ground_top = atlas.add_texture(Rect {
            min: Vec2::new(0.0, 80.0),
            max: Vec2::new(16.0, 96.0),
        });
        assets.terrain_indices.ground = atlas.add_texture(Rect {
            min: Vec2::new(0.0, 96.0),
            max: Vec2::new(16.0, 112.0),
        });

        assets.terrain_atlas = atlases.add(atlas);
    }

    // Set repeat address mode on tiling textures.
    if let Some(image) = images.get_mut(&assets.ground) {
        image.sampler_descriptor.address_mode_u = AddressMode::Repeat;
        image.sampler_descriptor.address_mode_v = AddressMode::Repeat;
    }
    if let Some(image) = images.get_mut(&assets.ground_top) {
        image.sampler_descriptor.address_mode_u = AddressMode::Repeat;
        image.sampler_descriptor.address_mode_v = AddressMode::Repeat;
    }
}
