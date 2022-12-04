use bevy::prelude::*;
use bevy_egui::EguiContext;
use noise::{NoiseFn, Perlin};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<CameraShake>()
            .init_resource::<StoredNoise>()
            .add_system(update_camera_shake);

        if cfg!(debug_assertions) {
            app.add_system(check_camera_shake_input.before(update_camera_shake));
        }
    }
}

pub struct StoredNoise {
    rotation_noise: Perlin,
    offset_noise_x: Perlin,
    offset_noise_y: Perlin,
}

impl Default for StoredNoise {
    fn default() -> Self {
        Self {
            rotation_noise: Perlin::new(fastrand::u32(..)),
            offset_noise_x: Perlin::new(fastrand::u32(..)),
            offset_noise_y: Perlin::new(fastrand::u32(..)),
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CameraShake {
    pub trauma: f32,
    /// The number of seconds for trauma to fully decay.
    pub decay: f32,
    pub max_angle: f32,
    pub max_offset: f32,
    pub noise_scale: f32,
}

impl Default for CameraShake {
    fn default() -> Self {
        Self {
            trauma: 0.0,
            decay: 1.0,
            max_angle: 0.0,
            max_offset: 0.0,
            noise_scale: 10.0,
        }
    }
}

impl CameraShake {
    fn decay_trauma(&mut self, dt: f32) {
        self.trauma -= dt / self.decay;
        self.trauma = self.trauma.clamp(0.0, 1.0);
    }

    pub fn add_trauma(&mut self, to_add: f32) {
        self.trauma += to_add;
        self.trauma = self.trauma.clamp(0.0, 1.0);
    }

    pub fn get_shake_value(&self) -> f32 {
        self.trauma.powi(2)
    }
}

fn update_camera_shake(
    time: Res<Time>,
    stored_noise: Res<StoredNoise>,
    mut camera_q: Query<(&mut Transform, &mut CameraShake)>,
) {
    let secs = time.seconds_since_startup();
    let dt = time.delta_seconds();
    for (mut transform, mut shake) in camera_q.iter_mut() {
        if shake.trauma > 0.0 {
            let shake_value = shake.get_shake_value();
            let noise_scale = shake.noise_scale as f64;

            let rotation_mult = stored_noise.rotation_noise.get([0.0, secs * noise_scale]);
            let angle = shake.max_angle * shake_value * rotation_mult as f32;
            transform.rotation = Quat::from_rotation_z(angle.to_radians());

            let offset_x_mult = stored_noise.offset_noise_x.get([0.0, secs * noise_scale]);
            let offset_y_mult = stored_noise.offset_noise_y.get([0.0, secs * noise_scale]);
            transform.translation.x = shake.max_offset * shake_value * offset_x_mult as f32;
            transform.translation.y = shake.max_offset * shake_value * offset_y_mult as f32;

            shake.decay_trauma(dt);
        } else {
            transform.translation.x = 0.0;
            transform.translation.y = 0.0;
            transform.rotation = Quat::IDENTITY;
        }
    }
}

fn check_camera_shake_input(
    keys: Res<Input<KeyCode>>,
    mut camera_q: Query<&mut CameraShake>,
    mut egui_ctx: ResMut<EguiContext>,
) {
    if egui_ctx.ctx_mut().wants_keyboard_input() {
        return;
    }

    let trauma_to_add = if keys.just_pressed(KeyCode::Key1) {
        0.2
    } else if keys.just_pressed(KeyCode::Key2) {
        0.5
    } else if keys.just_pressed(KeyCode::Key3) {
        1.0
    } else {
        return;
    };

    for mut shake in camera_q.iter_mut() {
        shake.add_trauma(trauma_to_add);
    }
}
