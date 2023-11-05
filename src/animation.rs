use std::time::Duration;

use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_asset::<Animation>()
            .add_systems(Update, animate_sprites);
    }
}

#[derive(TypePath, TypeUuid, Deref)]
#[uuid = "ae6a74db-f6fa-43c4-ac16-01d13b50e4c6"]
pub struct Animation(benimator::Animation);

impl Animation {
    pub fn from_indices(indices: impl IntoIterator<Item = usize>, frame_time: Duration) -> Self {
        Self(benimator::Animation::from_indices(indices, benimator::FrameRate::from_frame_duration(frame_time)))
    }
}

#[derive(Default, Component, Deref, DerefMut)]
pub struct AnimationState(benimator::State);

#[derive(Default, Component)]
pub struct Play;

pub fn animate_sprites(
    time: Res<Time>,
    animations: Res<Assets<Animation>>,
    mut query: Query<
        (
            &mut AnimationState,
            &mut TextureAtlasSprite,
            &Handle<Animation>,
        ),
        With<Play>,
    >,
) {
    for (mut player, mut texture, anim_handle) in query.iter_mut() {
        // Get the animation from the handle.
        let animation = match animations.get(anim_handle) {
            Some(anim) => anim,
            None => continue,
        };

        // Update the animation state.
        player.update(animation, time.delta());

        // Update the sprite's index into the texture atlas.
        texture.index = player.frame_index();
    }
}
