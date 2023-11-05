use std::time::Duration;

use bevy::prelude::*;
use bevy::{
    math::Rect,
    render::{
        texture::ImageSampler,
        render_resource::{AddressMode, SamplerDescriptor},
    },
};
use bevy_asset_loader::prelude::*;

use crate::{
    AppState,
    animation::Animation,
};

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_loading_state(
                LoadingState::new(AppState::Loading)
                    .continue_to_state(AppState::MainMenu)
            )
            .add_collection_to_loading_state::<_, GameAssets>(AppState::Loading)
            .add_systems(OnExit(AppState::Loading), assets_loaded);
    }
}

#[derive(Resource, AssetCollection)]
pub struct GameAssets {
    #[asset(path = "fonts/Kenney Blocks.ttf")]
    pub font: Handle<Font>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 4, rows = 1))]
    #[asset(path = "sprites/Player/bird1.png")]
    pub bird_atlas: Handle<TextureAtlas>,
    pub bird_anim: Handle<Animation>,

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
    mut animations: ResMut<Assets<Animation>>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut images: ResMut<Assets<Image>>,
) {
    debug!("Loaded assets!");

    // Bird anim info asset.
    let bird_anim = Animation::from_indices(0..=3, Duration::from_millis(150));
    assets.bird_anim = animations.add(bird_anim);

    // Populate terrain texture atlas.
    if let Some(image) = images.get_mut(&assets.terrain_image) {
        image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            ..default()
        });

        let mut atlas = TextureAtlas::new_empty(assets.terrain_image.clone(), image.size());
        assets.terrain_indices.pipe_bottom = atlas.add_texture(Rect {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(32.0, 16.0),
        });
        assets.terrain_indices.pipe_top = atlas.add_texture(Rect {
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
        image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            ..default()
        });
    }
    if let Some(image) = images.get_mut(&assets.ground_top) {
        image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            ..default()
        });
    }
}
