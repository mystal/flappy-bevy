use bevy::prelude::*;
use bevy_egui::EguiContext;
use heron::prelude::*;
use iyes_loopless::prelude::*;

use crate::{WINDOW_SIZE, AppState};

const CAMERA_POSITION: (f32, f32) = (WINDOW_SIZE.0 / 2.0, WINDOW_SIZE.1 / 2.0);

// Bird constants
const BIRD_RADIUS: f32 = 20.0;
const BIRD_GRAVITY: f32 = -1200.0;
const BIRD_MAX_FALL_SPEED: f32 = -800.0;
const BIRD_JUMP_SPEED: f32 = 450.0;

// Pipe constants
const PIPE_SPEED: f32 = 200.0;
const PIPE_START_X: f32 = 420.0;
const PIPE_END_X: f32 = -60.0;
const PIPE_GAP: f32 = 140.0;
const PIPE_WIDTH: f32 = 80.0;
const PIPE_SEGMENT_HEIGHT: f32 = 600.0;

// Ground constants
const GROUND_OFFSET: f32 = 40.0;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_enter_system(AppState::InGame, setup_game)
            .add_system(bird_movement.run_in_state(AppState::InGame).label("bird_movement"))
            .add_system(pipe_movement.run_in_state(AppState::InGame).before("bird_movement"));

        if cfg!(debug_assertions) {
            app.add_system(camera_control.run_in_state(AppState::InGame));
        }
    }
}

#[derive(Default, Component)]
struct Bird {
    speed: f32,
}

#[derive(Bundle)]
struct BirdBundle {
    bird: Bird,
    name: Name,
    transform: Transform,
    global_transform: GlobalTransform,
    rigid_body: RigidBody,
    collision_shape: CollisionShape,
}

impl BirdBundle {
    fn new(pos: Vec2) -> Self {
        Self {
            bird: Bird::default(),
            name: Name::new("Bird"),
            transform: Transform::from_translation(pos.extend(0.0)),
            global_transform: GlobalTransform::default(),
            rigid_body: RigidBody::KinematicPositionBased,
            collision_shape: CollisionShape::Sphere {
                radius: BIRD_RADIUS,
            },
        }
    }
}

#[derive(Component)]
struct Pipe;

#[derive(Bundle)]
struct PipeBundle {
    pipe: Pipe,
    name: Name,
    transform: Transform,
    global_transform: GlobalTransform,
    rigid_body: RigidBody,
}

impl PipeBundle {
    fn new(pos: Vec2) -> Self {
        Self {
            pipe: Pipe,
            name: Name::new("Pipe"),
            transform: Transform::from_translation(pos.extend(0.0)),
            global_transform: GlobalTransform::default(),
            rigid_body: RigidBody::KinematicPositionBased,
        }
    }
}

#[derive(Bundle)]
struct PipeScoreBundle {
    transform: Transform,
    global_transform: GlobalTransform,
    collision_shape: CollisionShape,
}

impl PipeScoreBundle {
    fn new(horizontal_offset: f32) -> Self {
        Self {
            transform: Transform::from_translation(Vec3::new(horizontal_offset, 0.0, 0.0)),
            global_transform: GlobalTransform::default(),
            collision_shape: CollisionShape::Cuboid {
                half_extends: Vec3::new(20.0, PIPE_GAP / 2.0, 0.0),
                border_radius: None,
            },
        }
    }
}

#[derive(Bundle)]
struct PipeSegmentBundle {
    transform: Transform,
    global_transform: GlobalTransform,
    collision_shape: CollisionShape,
}

impl PipeSegmentBundle {
    fn new(vertical_offset: f32) -> Self {
        Self {
            transform: Transform::from_translation(Vec3::new(0.0, vertical_offset, 0.0)),
            global_transform: GlobalTransform::default(),
            collision_shape: CollisionShape::Cuboid {
                half_extends: Vec3::new(PIPE_WIDTH / 2.0, PIPE_SEGMENT_HEIGHT / 2.0, 0.0),
                border_radius: None,
            },
        }
    }
}

fn setup_game(
    mut commands: Commands,
) {
    eprintln!("Setting up game");

    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.transform.translation.x = CAMERA_POSITION.0;
    camera_bundle.transform.translation.y = CAMERA_POSITION.1;
    commands.spawn_bundle(camera_bundle);

    // Spawn Bird
    commands.spawn_bundle(BirdBundle::new(Vec2::new(60.0, WINDOW_SIZE.1 / 2.0)));

    // Spawn ground sprite
    let ground_sprite = SpriteBundle {
        transform: Transform::from_translation(Vec3::new(WINDOW_SIZE.0 / 2.0, GROUND_OFFSET, 10.0)),
        sprite: Sprite {
            color: Color::DARK_GREEN,
            custom_size: Some(Vec2::new(WINDOW_SIZE.0, GROUND_OFFSET * 2.0)),
            ..default()
        },
        ..default()
    };
    commands.spawn_bundle(ground_sprite)
        .insert(Name::new("Ground"));

    // Spawn pipes
    commands.spawn_bundle(PipeBundle::new(Vec2::new(200.0, WINDOW_SIZE.1 / 2.0)))
        .with_children(|parent| {
            // Score detection
            parent.spawn_bundle(PipeScoreBundle::new(40.0));

            // Top pipe
            parent.spawn_bundle(PipeSegmentBundle::new((PIPE_SEGMENT_HEIGHT + PIPE_GAP) / 2.0));

            // Bottom pipe
            parent.spawn_bundle(PipeSegmentBundle::new(-(PIPE_SEGMENT_HEIGHT + PIPE_GAP) / 2.0));
        });
    commands.spawn_bundle(PipeBundle::new(Vec2::new(200.0 + 240.0, WINDOW_SIZE.1 / 2.0)))
        .with_children(|parent| {
            // Score detection
            parent.spawn_bundle(PipeScoreBundle::new(40.0));

            // Top pipe
            parent.spawn_bundle(PipeSegmentBundle::new((PIPE_SEGMENT_HEIGHT + PIPE_GAP) / 2.0));

            // Bottom pipe
            parent.spawn_bundle(PipeSegmentBundle::new(-(PIPE_SEGMENT_HEIGHT + PIPE_GAP) / 2.0));
        });
}

fn bird_movement(
    keys: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
    time: Res<Time>,
    mut bird_q: Query<(&mut Bird, &mut Transform)>,
    mut egui_ctx: ResMut<EguiContext>,
) {
    let jumped = (!egui_ctx.ctx_mut().wants_keyboard_input() && keys.just_pressed(KeyCode::Space)) ||
        (!egui_ctx.ctx_mut().wants_pointer_input() && buttons.just_pressed(MouseButton::Left));

    for (mut bird, mut transform) in bird_q.iter_mut() {
        // Update velocity.
        if jumped {
            bird.speed = BIRD_JUMP_SPEED;
        } else {
            bird.speed += BIRD_GRAVITY * time.delta_seconds();
            bird.speed = bird.speed.max(BIRD_MAX_FALL_SPEED);
        }

        transform.translation.y += bird.speed * time.delta_seconds();

        // Zero out speed if hitting the top of the screen.
        if transform.translation.y > WINDOW_SIZE.1 - BIRD_RADIUS {
            bird.speed = 0.0;
        }

        // Clamp position.
        transform.translation.y = transform.translation.y.clamp(GROUND_OFFSET * 2.0, WINDOW_SIZE.1 - BIRD_RADIUS);
    }
}

fn pipe_movement(
    time: Res<Time>,
    mut pipe_q: Query<&mut Transform, With<Pipe>>,
) {
    for mut transform in pipe_q.iter_mut() {
        transform.translation.x -= PIPE_SPEED * time.delta_seconds();

        // If scrolled past the left end of the screen, teleport to the right side.
        if transform.translation.x < PIPE_END_X {
            transform.translation.x = PIPE_START_X;
        }
    }
}

fn camera_control(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut egui_ctx: ResMut<EguiContext>,
    mut camera_q: Query<&mut Transform, With<Camera>>,
) {
    const CAMERA_MOVE_SPEED: f32 = 300.0;

    if egui_ctx.ctx_mut().wants_keyboard_input() {
        return;
    }

    let mut camera_transform = camera_q.single_mut();

    if keys.just_pressed(KeyCode::Key0) {
        camera_transform.translation.x = CAMERA_POSITION.0;
        camera_transform.translation.y = CAMERA_POSITION.1;
    }

    let move_dir = {
        let x = -(keys.pressed(KeyCode::A) as i8) + (keys.pressed(KeyCode::D) as i8);
        let y = -(keys.pressed(KeyCode::S) as i8) + (keys.pressed(KeyCode::W) as i8);
        Vec2::new(x as f32, y as f32)
    };

    camera_transform.translation += move_dir.extend(0.0) * CAMERA_MOVE_SPEED * time.delta_seconds();
}
