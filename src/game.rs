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
const PIPE_SPACING: f32 = 240.0;
const PIPE_INIT_X: f32 = 400.0;

// Ground constants
const GROUND_OFFSET: f32 = 40.0;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_loopless_state(GameState::Ready)
            .add_enter_system(AppState::InGame, setup_game)
            .add_enter_system(GameState::Ready, reset_bird)
            .add_enter_system(GameState::Ready, reset_pipes)
            .add_enter_system(GameState::Playing, enter_playing)
            .add_exit_system(GameState::Playing, exit_playing)
            .add_enter_system(GameState::Lost, enter_lost)
            .add_system(check_start_input.run_in_state(AppState::InGame).run_in_state(GameState::Ready))
            .add_system(check_reset_input.run_in_state(AppState::InGame).run_in_state(GameState::Lost))
            .add_system(bird_movement.run_in_state(AppState::InGame).label("bird_movement"))
            .add_system(pipe_movement.run_in_state(AppState::InGame).run_in_state(GameState::Playing).before("bird_movement"))
            .add_system(check_bird_scored.run_in_state(AppState::InGame).run_in_state(GameState::Playing).after("bird_movement"))
            .add_system(check_bird_crashed.run_in_state(AppState::InGame).run_in_state(GameState::Playing).after("bird_movement"));

        if cfg!(debug_assertions) {
            app.add_system(camera_control.run_in_state(AppState::InGame));
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    Ready,
    Playing,
    Lost,
}

#[derive(Default)]
struct GameData {
    score: u16,
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

#[derive(Component)]
struct PipeScoreZone;

#[derive(Bundle)]
struct PipeScoreBundle {
    score_zone: PipeScoreZone,
    transform: Transform,
    global_transform: GlobalTransform,
    collision_shape: CollisionShape,
}

impl PipeScoreBundle {
    fn new(horizontal_offset: f32) -> Self {
        Self {
            score_zone: PipeScoreZone,
            transform: Transform::from_translation(Vec3::new(horizontal_offset, 0.0, 0.0)),
            global_transform: GlobalTransform::default(),
            collision_shape: CollisionShape::Cuboid {
                half_extends: Vec3::new(20.0, PIPE_GAP / 2.0, 0.0),
                border_radius: None,
            },
        }
    }
}

#[derive(Component)]
struct PipeSegment;

#[derive(Bundle)]
struct PipeSegmentBundle {
    segment: PipeSegment,
    transform: Transform,
    global_transform: GlobalTransform,
    collision_shape: CollisionShape,
}

impl PipeSegmentBundle {
    fn new(vertical_offset: f32) -> Self {
        Self {
            segment: PipeSegment,
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

    commands.insert_resource(GameData::default());

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

    // Spawn pipes offscreen.
    commands.spawn_bundle(PipeBundle::new(Vec2::new(PIPE_INIT_X, WINDOW_SIZE.1 / 2.0)))
        .with_children(|parent| {
            // Score detection
            parent.spawn_bundle(PipeScoreBundle::new(40.0));

            // Top pipe
            parent.spawn_bundle(PipeSegmentBundle::new((PIPE_SEGMENT_HEIGHT + PIPE_GAP) / 2.0));

            // Bottom pipe
            parent.spawn_bundle(PipeSegmentBundle::new(-(PIPE_SEGMENT_HEIGHT + PIPE_GAP) / 2.0));
        });
    commands.spawn_bundle(PipeBundle::new(Vec2::new(PIPE_INIT_X + PIPE_SPACING, WINDOW_SIZE.1 / 2.0)))
        .with_children(|parent| {
            // Score detection
            parent.spawn_bundle(PipeScoreBundle::new(40.0));

            // Top pipe
            parent.spawn_bundle(PipeSegmentBundle::new((PIPE_SEGMENT_HEIGHT + PIPE_GAP) / 2.0));

            // Bottom pipe
            parent.spawn_bundle(PipeSegmentBundle::new(-(PIPE_SEGMENT_HEIGHT + PIPE_GAP) / 2.0));
        });

    // Make sure we're in the Ready state.
    commands.insert_resource(NextState(GameState::Ready));
}

fn reset_bird(
    app_state: Res<CurrentState<AppState>>,
    mut game_data: ResMut<GameData>,
    mut bird_q: Query<(&mut Bird, &mut Transform)>,
) {
    if app_state.0 != AppState::InGame {
        return;
    }

    eprintln!("reset_bird");

    game_data.score = 0;

    for (mut bird, mut transform) in bird_q.iter_mut() {
        bird.speed = 0.0;
        transform.translation = Vec3::new(60.0, WINDOW_SIZE.1 / 2.0, 0.0);
    }
}

fn reset_pipes(
    app_state: Res<CurrentState<AppState>>,
    mut pipe_q: Query<&mut Transform, With<Pipe>>,
) {
    if app_state.0 != AppState::InGame {
        return;
    }

    eprintln!("reset_pipes");

    for (i, mut transform) in pipe_q.iter_mut().enumerate() {
        transform.translation = Vec3::new(PIPE_INIT_X + (i as f32 * PIPE_SPACING), WINDOW_SIZE.1 / 2.0, 0.0);
    }
}

fn enter_playing(
    mut bird_q: Query<&mut Bird>,
) {
    eprintln!("Enter Playing");

    for mut bird in bird_q.iter_mut() {
        bird.speed = BIRD_JUMP_SPEED;
    }
}

fn exit_playing(
    mut bird_q: Query<&mut Bird>,
) {
    for mut bird in bird_q.iter_mut() {
        bird.speed = 0.0;
    }
}

fn enter_lost() {
    eprintln!("Enter Lost");
}

fn check_start_input(
    keys: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
    mut egui_ctx: ResMut<EguiContext>,
    mut commands: Commands,
) {
    let start_pressed = (!egui_ctx.ctx_mut().wants_keyboard_input() && keys.just_pressed(KeyCode::Space)) ||
        (!egui_ctx.ctx_mut().wants_pointer_input() && buttons.just_pressed(MouseButton::Left));
    if start_pressed {
        commands.insert_resource(NextState(GameState::Playing));
    }
}

fn check_reset_input(
    keys: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
    mut egui_ctx: ResMut<EguiContext>,
    mut commands: Commands,
) {
    let start_pressed = (!egui_ctx.ctx_mut().wants_keyboard_input() && keys.just_pressed(KeyCode::Space)) ||
        (!egui_ctx.ctx_mut().wants_pointer_input() && buttons.just_pressed(MouseButton::Left));
    if start_pressed {
        commands.insert_resource(NextState(GameState::Ready));
    }
}

fn bird_movement(
    game_state: Res<CurrentState<GameState>>,
    keys: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
    time: Res<Time>,
    mut bird_q: Query<(&mut Bird, &mut Transform)>,
    mut egui_ctx: ResMut<EguiContext>,
) {
    if game_state.0 == GameState::Ready {
        return;
    }

    let jumped = game_state.0 == GameState::Playing &&
        ((!egui_ctx.ctx_mut().wants_keyboard_input() && keys.just_pressed(KeyCode::Space)) ||
        (!egui_ctx.ctx_mut().wants_pointer_input() && buttons.just_pressed(MouseButton::Left)));

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

fn check_bird_scored(
    mut collisions: EventReader<CollisionEvent>,
    mut game_data: ResMut<GameData>,
    bird_q: Query<(), With<Bird>>,
    pipe_score_q: Query<(), With<PipeScoreZone>>,
) {
    let bird_entered_score_zone = |entity1, entity2| {
        bird_q.contains(entity1) && pipe_score_q.contains(entity2)
    };
    for event in collisions.iter() {
        if let CollisionEvent::Started(data1, data2) = event {
            let entity1 = data1.collision_shape_entity();
            let entity2 = data2.collision_shape_entity();
            let scored = bird_entered_score_zone(entity1, entity2) || bird_entered_score_zone(entity2, entity1);
            if scored {
                game_data.score += 1;
                println!("Scored! {}", game_data.score);
            }
        }
    }
}

fn check_bird_crashed(
    mut collisions: EventReader<CollisionEvent>,
    bird_q: Query<&Transform, With<Bird>>,
    pipe_segment_q: Query<(), With<PipeSegment>>,
    mut commands: Commands,
) {
    // Check if bird hit the ground.
    if let Ok(transform) = bird_q.get_single() {
        if transform.translation.y <= (GROUND_OFFSET * 2.0) + BIRD_RADIUS {
            commands.insert_resource(NextState(GameState::Lost));
            return;
        }
    }

    // Check if bird hit a pipe.
    let bird_hit_pipe = |entity1, entity2| {
        bird_q.contains(entity1) && pipe_segment_q.contains(entity2)
    };
    for event in collisions.iter() {
        if let CollisionEvent::Started(data1, data2) = event {
            let entity1 = data1.collision_shape_entity();
            let entity2 = data2.collision_shape_entity();
            let hit_pipe = bird_hit_pipe(entity1, entity2) || bird_hit_pipe(entity2, entity1);
            if hit_pipe {
                commands.insert_resource(NextState(GameState::Lost));
            }
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
