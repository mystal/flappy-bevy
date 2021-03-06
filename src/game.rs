use bevy::prelude::*;
use bevy::{
    render::{camera::WindowOrigin, mesh::VertexAttributeValues},
    sprite::Anchor,
};
use bevy_egui::EguiContext;
use heron::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    GAME_SIZE, AppState,
    assets::GameAssets,
    camera::CameraShake,
    window::WindowScale,
};

// Bird constants
const BIRD_RADIUS: f32 = 7.0;
const BIRD_OFFSET_X: f32 = 30.0;
const BIRD_GRAVITY: f32 = -650.0;
const BIRD_MAX_FALL_SPEED: f32 = -400.0;
const BIRD_JUMP_SPEED: f32 = 230.0;

// Pipe constants
const PIPE_SPEED: f32 = 80.0;
const PIPE_START_X: f32 = 210.0;
const PIPE_END_X: f32 = -30.0;
const PIPE_GAP: f32 = 70.0;
const PIPE_WIDTH: f32 = 28.0;
const PIPE_SEGMENT_HEIGHT: f32 = 300.0;
const PIPE_ENTRANCE_WIDTH: f32 = 32.0;
const PIPE_ENTRANCE_HEIGHT: f32 = 16.0;
const PIPE_SPACING: f32 = 120.0;
const PIPE_INIT_X: f32 = 200.0;
const PIPE_Y_RAND_RANGE: f32 = 60.0;

// Ground constants
const GROUND_OFFSET: f32 = (GAME_SIZE.1 - 256.0) / 2.0;

// Z values
const BIRD_Z: f32 = 15.0;
const PIPE_Z: f32 = 4.0;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_loopless_state(GameState::Ready)
            .add_event::<TapEvent>()
            .insert_resource(GameData::default())
            .add_enter_system(AppState::InGame, setup_game)
            .add_enter_system(GameState::Ready, reset_bird)
            .add_enter_system(GameState::Ready, reset_pipes)
            .add_enter_system(GameState::Playing, enter_playing)
            .add_exit_system(GameState::Playing, exit_playing)
            .add_enter_system(GameState::Lost, enter_lost)
            .add_system(check_tap_input.run_in_state(AppState::InGame).label("check_tap_input"))
            .add_system(check_state_transition.run_in_state(AppState::InGame).run_not_in_state(GameState::Playing).after("check_tap_input"))
            .add_system(bird_movement.run_in_state(AppState::InGame).label("bird_movement").after("check_tap_input"))
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
struct TapEvent;

#[derive(Default)]
struct GameData {
    score: u16,
    score_text: Option<Entity>,
    last_pipe_y: f32,
}

impl GameData {
    fn gen_random_pipe_y(&mut self) -> f32 {
        const MIN: f32 = (GROUND_OFFSET * 2.0) + (PIPE_GAP * 0.75);
        const MAX: f32 = GAME_SIZE.1 - (PIPE_GAP * 0.75);

        let range_min = (self.last_pipe_y - PIPE_Y_RAND_RANGE).max(MIN);
        let range_max = (self.last_pipe_y + PIPE_Y_RAND_RANGE).min(MAX);

        let multiplier = fastrand::f32();
        self.last_pipe_y = range_min + (range_max - range_min) * multiplier;
        // Round the position so that pipes are on integers and their sprites render properly.
        self.last_pipe_y = self.last_pipe_y.round();

        trace!(self.last_pipe_y);

        self.last_pipe_y
    }
}

#[derive(Default, Component)]
struct Bird {
    speed: f32,
    angle: f32,
}

#[derive(Bundle)]
struct BirdBundle {
    bird: Bird,
    name: Name,
    #[bundle]
    sprite_sheet: SpriteSheetBundle,
    rigid_body: RigidBody,
    collision_shape: CollisionShape,
}

impl BirdBundle {
    fn new(pos: Vec2, texture_atlas: Handle<TextureAtlas>) -> Self {
        let sprite_sheet = SpriteSheetBundle {
            texture_atlas,
            transform: Transform::from_translation(pos.extend(BIRD_Z)),
            ..default()
        };
        Self {
            bird: Bird::default(),
            name: Name::new("Bird"),
            sprite_sheet,
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
                half_extends: Vec3::new(10.0, PIPE_GAP / 2.0, 0.0),
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
    #[bundle]
    sprite_bundle: SpriteBundle,
    collision_shape: CollisionShape,
}

impl PipeSegmentBundle {
    fn new(vertical_offset: f32, texture: Handle<Image>) -> Self {
        Self {
            segment: PipeSegment,
            sprite_bundle: SpriteBundle {
                transform: Transform::from_translation(Vec3::new(0.0, vertical_offset, PIPE_Z)),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(PIPE_WIDTH, PIPE_SEGMENT_HEIGHT)),
                    ..default()
                },
                texture,
                ..default()
            },
            collision_shape: CollisionShape::Cuboid {
                half_extends: Vec3::new(PIPE_WIDTH / 2.0, PIPE_SEGMENT_HEIGHT / 2.0, 0.0),
                border_radius: None,
            },
        }
    }
}

#[derive(Bundle)]
struct PipeEntranceBundle {
    segment: PipeSegment,
    #[bundle]
    sprite_sheet: SpriteSheetBundle,
    collision_shape: CollisionShape,
}

impl PipeEntranceBundle {
    fn new(vertical_offset: f32, sprite_index: usize, texture_atlas: Handle<TextureAtlas>) -> Self {
        Self {
            segment: PipeSegment,
            sprite_sheet: SpriteSheetBundle {
                transform: Transform::from_translation(Vec3::new(0.0, vertical_offset, PIPE_Z + 1.0)),
                sprite: TextureAtlasSprite {
                    index: sprite_index,
                    ..default()
                },
                texture_atlas,
                ..default()
            },
            collision_shape: CollisionShape::Cuboid {
                half_extends: Vec3::new(PIPE_ENTRANCE_WIDTH / 2.0, PIPE_ENTRANCE_HEIGHT / 2.0, 0.0),
                border_radius: None,
            },
        }
    }
}

fn spawn_pipe(
    commands: &mut Commands,
    assets: &GameAssets,
    game_data: &mut GameData,
    pipe_index: u8,
) {
    commands.spawn_bundle(PipeBundle::new(Vec2::new(PIPE_INIT_X * pipe_index as f32, game_data.gen_random_pipe_y())))
        .with_children(|parent| {
            // Score detection
            parent.spawn_bundle(PipeScoreBundle::new(20.0));

            // Top pipe
            parent.spawn_bundle(PipeSegmentBundle::new((PIPE_SEGMENT_HEIGHT + PIPE_GAP) / 2.0, assets.pipe_center.clone()));
            parent.spawn_bundle(PipeEntranceBundle::new((PIPE_ENTRANCE_HEIGHT + PIPE_GAP) / 2.0, assets.terrain_indices.pipe_top, assets.terrain_atlas.clone()));

            // Bottom pipe
            parent.spawn_bundle(PipeSegmentBundle::new(-(PIPE_SEGMENT_HEIGHT + PIPE_GAP) / 2.0, assets.pipe_center.clone()));
            parent.spawn_bundle(PipeEntranceBundle::new(-(PIPE_ENTRANCE_HEIGHT + PIPE_GAP) / 2.0, assets.terrain_indices.pipe_bottom, assets.terrain_atlas.clone()));
        });
}

fn setup_game(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut game_data: ResMut<GameData>,
    window_scale: Res<WindowScale>,
    images: Res<Assets<Image>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    debug!("Setting up game");

    // Spawn an orthographic camera rooted at the bottom left parented under a transform to support
    // camera shake.
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    // Make the projection origin the bottom left so the camera at 0,0 will have values increasing
    // up and to the right.
    camera_bundle.orthographic_projection.window_origin = WindowOrigin::BottomLeft;
    camera_bundle.orthographic_projection.scale = 1.0 / window_scale.0 as f32;
    let camera_entity = commands.spawn_bundle(camera_bundle)
        .insert(CameraShake {
            max_angle: 10.0,
            max_offset: 10.0,
            noise_scale: 15.0,
            ..default()
        })
        .id();
    commands.spawn_bundle(TransformBundle::default())
        .insert(Name::new("CameraParent"))
        .add_child(camera_entity);

    // Spawn Bird
    commands.spawn_bundle(BirdBundle::new(Vec2::new(BIRD_OFFSET_X, GAME_SIZE.1 / 2.0), assets.bird_atlas.clone()))
        .insert(assets.bird_anim.clone())
        .insert(benimator::Play);

    // Spawn background sprite.
    let background_sprite = SpriteBundle {
        // Positioned at the top of the camera view.
        transform: Transform::from_translation(Vec3::new(GAME_SIZE.0 / 2.0, GAME_SIZE.1, 0.0)),
        sprite: Sprite {
            anchor: Anchor::TopCenter,
            ..default()
        },
        texture: assets.background.clone(),
        ..default()
    };
    commands.spawn_bundle(background_sprite)
        .insert(Name::new("Background"));

    // Spawn tiling ground texture.
    let ground_image_size = images.get(&assets.ground).unwrap().size();
    let mut ground_mesh = Mesh::from(shape::Quad::default());
    if let Some(VertexAttributeValues::Float32x2(uvs)) = ground_mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
        for uv in uvs {
            uv[0] *= GAME_SIZE.0 / ground_image_size.x;
            uv[1] *= (GROUND_OFFSET * 2.0) / ground_image_size.y;
        }
    }
    let ground_transform = Transform {
        translation: Vec3::new(GAME_SIZE.0 / 2.0, GROUND_OFFSET, 10.0),
        scale: Vec3::new(GAME_SIZE.0, GROUND_OFFSET * 2.0, 1.0),
        ..default()
    };
    let ground_bundle = ColorMesh2dBundle {
        transform: ground_transform,
        material: materials.add(assets.ground.clone().into()),
        mesh: meshes.add(ground_mesh.into()).into(),
        ..default()
    };
    commands.spawn_bundle(ground_bundle)
        .insert(Name::new("Ground"));

    // Spawn tiling ground top texture.
    let ground_image_size = images.get(&assets.ground_top).unwrap().size();
    let mut ground_mesh = Mesh::from(shape::Quad::default());
    if let Some(VertexAttributeValues::Float32x2(uvs)) = ground_mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
        for uv in uvs {
            uv[0] *= GAME_SIZE.0 / ground_image_size.x;
        }
    }
    let ground_transform = Transform {
        translation: Vec3::new(GAME_SIZE.0 / 2.0, (GROUND_OFFSET * 2.0) - (ground_image_size.y / 2.0), 10.0),
        scale: Vec3::new(GAME_SIZE.0, ground_image_size.y, 1.0),
        ..default()
    };
    let ground_bundle = ColorMesh2dBundle {
        transform: ground_transform,
        material: materials.add(assets.ground_top.clone().into()),
        mesh: meshes.add(ground_mesh.into()).into(),
        ..default()
    };
    commands.spawn_bundle(ground_bundle)
        .insert(Name::new("Ground"));

    // Initialize last_pipe_y in the center of the screen so we start generating new locations
    // around it.
    game_data.last_pipe_y = GAME_SIZE.1 / 2.0;

    // Spawn pipes offscreen.
    spawn_pipe(&mut commands, &assets, &mut game_data, 0);
    spawn_pipe(&mut commands, &assets, &mut game_data, 1);

    // Create score text.
    let style = TextStyle {
        font: assets.font.clone(),
        font_size: 30.0,
        color: Color::WHITE,
    };
    let alignment = TextAlignment {
        horizontal: HorizontalAlign::Center,
        ..default()
    };
    let score_text_id = commands
        .spawn_bundle(Text2dBundle {
            text: Text::with_section("0", style.clone(), alignment),
            transform: Transform::from_translation(Vec3::new(GAME_SIZE.0 / 2.0, 300.0, 50.0)),
            ..default()
        })
        .insert(Name::new("Score Text"))
        .id();
    game_data.score_text = Some(score_text_id);

    // Make sure we're in the Ready state.
    commands.insert_resource(NextState(GameState::Ready));
}

fn reset_bird(
    mut commands: Commands,
    app_state: Res<CurrentState<AppState>>,
    mut game_data: ResMut<GameData>,
    mut bird_q: Query<(Entity, &mut Bird, &mut Transform)>,
    mut score_text_q: Query<&mut Text>,
) {
    if app_state.0 != AppState::InGame {
        return;
    }

    debug!("reset_bird");

    game_data.score = 0;
    if let Some(entity) = game_data.score_text {
        if let Ok(mut text) = score_text_q.get_mut(entity) {
            text.sections[0].value = game_data.score.to_string();
        }
    }

    for (entity, mut bird, mut transform) in bird_q.iter_mut() {
        bird.speed = 0.0;
        bird.angle = 0.0;
        transform.translation = Vec3::new(BIRD_OFFSET_X, GAME_SIZE.1 / 2.0, BIRD_Z);
        transform.rotation = Quat::IDENTITY;
        commands.entity(entity).insert(benimator::Play);
    }
}

fn reset_pipes(
    app_state: Res<CurrentState<AppState>>,
    mut game_data: ResMut<GameData>,
    mut pipe_q: Query<&mut Transform, With<Pipe>>,
) {
    if app_state.0 != AppState::InGame {
        return;
    }

    debug!("reset_pipes");

    game_data.last_pipe_y = GAME_SIZE.1 / 2.0;
    for (i, mut transform) in pipe_q.iter_mut().enumerate() {
        transform.translation = Vec3::new(PIPE_INIT_X + (i as f32 * PIPE_SPACING), game_data.gen_random_pipe_y(), 0.0);
    }
}

fn enter_playing(
    mut bird_q: Query<&mut Bird>,
) {
    debug!("Enter Playing");

    for mut bird in bird_q.iter_mut() {
        bird.speed = BIRD_JUMP_SPEED;
    }
}

fn exit_playing(
    mut commands: Commands,
    mut bird_q: Query<(Entity, &mut Bird)>,
) {
    for (entity, mut bird) in bird_q.iter_mut() {
        bird.speed = 0.0;
        commands.entity(entity).remove::<benimator::Play>();
    }
}

fn enter_lost(
    mut camera_q: Query<&mut CameraShake>,
) {
    debug!("Enter Lost");

    for mut shake in camera_q.iter_mut() {
        shake.add_trauma(0.4);
    }
}

fn check_tap_input(
    keys: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
    touches: Res<Touches>,
    mut egui_ctx: ResMut<EguiContext>,
    mut tap_events: EventWriter<TapEvent>,
) {
    let keyboard_input = !egui_ctx.ctx_mut().wants_keyboard_input() && keys.just_pressed(KeyCode::Space);
    let mouse_input = !egui_ctx.ctx_mut().wants_pointer_input() && buttons.just_pressed(MouseButton::Left);
    let touch_input = !egui_ctx.ctx_mut().wants_pointer_input() && touches.iter_just_pressed().count() > 0;
    if keyboard_input || mouse_input || touch_input {
        tap_events.send_default();
    }
}

fn check_state_transition(
    game_state: Res<CurrentState<GameState>>,
    mut tap_events: EventReader<TapEvent>,
    mut commands: Commands,
    bird_q: Query<&Transform, With<Bird>>,
) {
    // Making sure we drain the events.
    if tap_events.iter().next().is_none() {
        return;
    }

    match game_state.0 {
        GameState::Ready => commands.insert_resource(NextState(GameState::Playing)),
        GameState::Lost => {
            // Make sure the bird has hit the ground before resetting.
            if let Ok(bird_transform) = bird_q.get_single() {
                if bird_transform.translation.y <= GROUND_OFFSET * 2.0 {
                    commands.insert_resource(NextState(GameState::Ready));
                }
            }
        }
        _ => {}
    }
}

fn bird_movement(
    game_state: Res<CurrentState<GameState>>,
    tap_events: EventReader<TapEvent>,
    time: Res<Time>,
    mut bird_q: Query<(&mut Bird, &mut Transform)>,
) {
    if game_state.0 == GameState::Ready {
        return;
    }

    let dt = time.delta_seconds();

    let jumped = game_state.0 == GameState::Playing && !tap_events.is_empty();
    for (mut bird, mut transform) in bird_q.iter_mut() {
        // Update velocity.
        if jumped {
            bird.speed = BIRD_JUMP_SPEED;
        } else {
            // Fall with gravity.
            bird.speed += BIRD_GRAVITY * dt;
            bird.speed = bird.speed.max(BIRD_MAX_FALL_SPEED);
        }

        transform.translation.y += bird.speed * time.delta_seconds();

        // Zero out speed if hitting the top of the screen.
        if transform.translation.y > GAME_SIZE.1 - BIRD_RADIUS {
            bird.speed = 0.0;
        }

        // Clamp position.
        transform.translation.y = transform.translation.y.clamp(GROUND_OFFSET * 2.0, GAME_SIZE.1 - BIRD_RADIUS);

        // Set bird rotation based on speed.
        if bird.speed > 0.0 {
            // Rotate left.
            bird.angle += 600.0 * dt;
        } else if bird.speed < -110.0 {
            // Rotate right.
            bird.angle -= 480.0 * dt;
        }
        bird.angle = bird.angle.clamp(-90.0, 30.0);
        transform.rotation = Quat::from_rotation_z(bird.angle.to_radians());
    }
}

fn pipe_movement(
    time: Res<Time>,
    mut game_data: ResMut<GameData>,
    mut pipe_q: Query<&mut Transform, With<Pipe>>,
) {
    for mut transform in pipe_q.iter_mut() {
        transform.translation.x -= PIPE_SPEED * time.delta_seconds();

        // If scrolled past the left end of the screen, teleport to the right side.
        if transform.translation.x < PIPE_END_X {
            transform.translation.x = PIPE_START_X;
            transform.translation.y = game_data.gen_random_pipe_y();
        }
    }
}

fn check_bird_scored(
    mut collisions: EventReader<CollisionEvent>,
    mut game_data: ResMut<GameData>,
    bird_q: Query<(), With<Bird>>,
    pipe_score_q: Query<(), With<PipeScoreZone>>,
    mut score_text_q: Query<&mut Text>,
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
                if let Some(entity) = game_data.score_text {
                    if let Ok(mut text) = score_text_q.get_mut(entity) {
                        text.sections[0].value = game_data.score.to_string();
                    }
                }
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
        camera_transform.translation.x = 0.0;
        camera_transform.translation.y = 0.0;
    }

    let move_dir = {
        let x = -(keys.pressed(KeyCode::A) as i8) + (keys.pressed(KeyCode::D) as i8);
        let y = -(keys.pressed(KeyCode::S) as i8) + (keys.pressed(KeyCode::W) as i8);
        Vec2::new(x as f32, y as f32)
    };

    camera_transform.translation += move_dir.extend(0.0) * CAMERA_MOVE_SPEED * time.delta_seconds();
}
