use bevy::prelude::*;
use bevy::{
    render::mesh::VertexAttributeValues,
    sprite::Anchor,
};
use bevy_egui::EguiContexts;
use bevy_rapier2d::prelude::*;

use crate::{
    GAME_SIZE, AppState,
    animation,
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
const PIPE_BODY_HEIGHT: f32 = 300.0;
const PIPE_MOUTH_WIDTH: f32 = 32.0;
const PIPE_MOUTH_HEIGHT: f32 = 16.0;
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
            .add_state::<GameState>()
            .add_event::<TapEvent>()
            .insert_resource(GameData::default())

            // OnEnter/OnExit systems.
            .add_system(setup_game.in_schedule(OnEnter(AppState::InGame)))
            .add_systems((reset_bird, reset_pipes).in_schedule(OnEnter(GameState::Ready)))
            .add_system(enter_playing.in_schedule(OnEnter(GameState::Playing)))
            .add_system(exit_playing.in_schedule(OnExit(GameState::Playing)))
            .add_system(enter_lost.in_schedule(OnEnter(GameState::Lost)))

            // OnUpdate systems.
            .add_systems((
                check_tap_input,
                check_state_transition.run_if(not(in_state(GameState::Playing))).after(check_tap_input),
                bird_movement.after(check_tap_input),
                pipe_movement.run_if(in_state(GameState::Playing)).before(bird_movement),
                check_bird_scored.run_if(in_state(GameState::Playing)).after(bird_movement),
                check_bird_crashed.run_if(in_state(GameState::Playing)).after(bird_movement),
            ).in_set(OnUpdate(AppState::InGame)));

        if cfg!(debug_assertions) {
            app.add_system(camera_control.in_set(OnUpdate(AppState::InGame)));
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum GameState {
    #[default]
    Ready,
    Playing,
    Lost,
}

#[derive(Default)]
struct TapEvent;

#[derive(Default, Resource)]
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
    collision_shape: Collider,
    sensor: Sensor,
    active_collision_types: ActiveCollisionTypes,
    active_events: ActiveEvents,
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
            collision_shape: Collider::ball(BIRD_RADIUS),
            sensor: Sensor,
            active_collision_types: ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
            active_events: ActiveEvents::COLLISION_EVENTS,
        }
    }
}

#[derive(Component)]
struct Pipe;

#[derive(Bundle)]
struct PipeBundle {
    pipe: Pipe,
    name: Name,
    #[bundle]
    spatial: SpatialBundle,
    rigid_body: RigidBody,
}

impl PipeBundle {
    fn new(pos: Vec2) -> Self {
        let transform = Transform::from_translation(pos.extend(0.0));
        Self {
            pipe: Pipe,
            name: Name::new("Pipe"),
            spatial: SpatialBundle::from_transform(transform),
            rigid_body: RigidBody::KinematicPositionBased,
        }
    }
}

#[derive(Component)]
struct PipeScoreZone;

#[derive(Bundle)]
struct PipeScoreBundle {
    score_zone: PipeScoreZone,
    name: Name,
    #[bundle]
    transform: TransformBundle,
    collision_shape: Collider,
    sensor: Sensor,
    active_collision_types: ActiveCollisionTypes,
}

impl PipeScoreBundle {
    fn new(horizontal_offset: f32) -> Self {
        let transform = Transform::from_translation(Vec3::new(horizontal_offset, 0.0, 0.0));
        Self {
            score_zone: PipeScoreZone,
            name: "ScoreZone".into(),
            transform: TransformBundle::from_transform(transform),
            collision_shape: Collider::cuboid(10.0, PIPE_GAP / 2.0),
            sensor: Sensor,
            active_collision_types: ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
        }
    }
}

#[derive(Component)]
struct PipeBody;

#[derive(Bundle)]
struct PipeBodyBundle {
    body: PipeBody,
    name: Name,
    #[bundle]
    sprite_bundle: SpriteBundle,
    collision_shape: Collider,
    sensor: Sensor,
    active_collision_types: ActiveCollisionTypes,
}

impl PipeBodyBundle {
    fn new(vertical_offset: f32, texture: Handle<Image>) -> Self {
        Self {
            body: PipeBody,
            name: "PipeBody".into(),
            sprite_bundle: SpriteBundle {
                transform: Transform::from_translation(Vec3::new(0.0, vertical_offset, PIPE_Z)),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(PIPE_WIDTH, PIPE_BODY_HEIGHT)),
                    ..default()
                },
                texture,
                ..default()
            },
            collision_shape: Collider::cuboid(PIPE_WIDTH / 2.0, PIPE_BODY_HEIGHT / 2.0),
            sensor: Sensor,
            active_collision_types: ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
        }
    }
}

#[derive(Bundle)]
struct PipeMouthBundle {
    body: PipeBody,
    name: Name,
    #[bundle]
    sprite_sheet: SpriteSheetBundle,
    collision_shape: Collider,
    sensor: Sensor,
    active_collision_types: ActiveCollisionTypes,
}

impl PipeMouthBundle {
    fn new(vertical_offset: f32, sprite_index: usize, texture_atlas: Handle<TextureAtlas>) -> Self {
        Self {
            body: PipeBody,
            name: "PipeMouth".into(),
            sprite_sheet: SpriteSheetBundle {
                transform: Transform::from_translation(Vec3::new(0.0, vertical_offset, PIPE_Z + 1.0)),
                sprite: TextureAtlasSprite {
                    index: sprite_index,
                    ..default()
                },
                texture_atlas,
                ..default()
            },
            collision_shape: Collider::cuboid(PIPE_MOUTH_WIDTH / 2.0, PIPE_MOUTH_HEIGHT / 2.0),
            sensor: Sensor,
            active_collision_types: ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
        }
    }
}

fn spawn_pipe(
    commands: &mut Commands,
    assets: &GameAssets,
    game_data: &mut GameData,
    pipe_index: u8,
) {
    commands.spawn(PipeBundle::new(Vec2::new(PIPE_INIT_X * pipe_index as f32, game_data.gen_random_pipe_y())))
        .with_children(|parent| {
            // Score detection
            parent.spawn(PipeScoreBundle::new(20.0));

            // Top pipe
            parent.spawn(PipeBodyBundle::new((PIPE_BODY_HEIGHT + PIPE_GAP) / 2.0, assets.pipe_center.clone()));
            parent.spawn(PipeMouthBundle::new((PIPE_MOUTH_HEIGHT + PIPE_GAP) / 2.0, assets.terrain_indices.pipe_top, assets.terrain_atlas.clone()));

            // Bottom pipe
            parent.spawn(PipeBodyBundle::new(-(PIPE_BODY_HEIGHT + PIPE_GAP) / 2.0, assets.pipe_center.clone()));
            parent.spawn(PipeMouthBundle::new(-(PIPE_MOUTH_HEIGHT + PIPE_GAP) / 2.0, assets.terrain_indices.pipe_bottom, assets.terrain_atlas.clone()));
        });
}

fn setup_game(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
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
    let mut camera_bundle = Camera2dBundle::default();
    // Make the projection origin the bottom left so the camera at 0,0 will have values increasing
    // up and to the right.
    camera_bundle.projection.viewport_origin = Vec2::ZERO;
    camera_bundle.projection.scale = 1.0 / window_scale.0 as f32;
    let camera_entity = commands.spawn(camera_bundle)
        .insert(CameraShake {
            max_angle: 10.0,
            max_offset: 10.0,
            noise_scale: 15.0,
            ..default()
        })
        .id();
    commands.spawn(TransformBundle::default())
        .insert(Name::new("CameraParent"))
        .add_child(camera_entity);

    // Spawn Bird
    commands.spawn(BirdBundle::new(Vec2::new(BIRD_OFFSET_X, GAME_SIZE.1 / 2.0), assets.bird_atlas.clone()))
        .insert(assets.bird_anim.clone())
        .insert(animation::AnimationState::default())
        .insert(animation::Play);

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
    commands.spawn(background_sprite)
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
    commands.spawn(ground_bundle)
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
        translation: Vec3::new(GAME_SIZE.0 / 2.0, (GROUND_OFFSET * 2.0) - (ground_image_size.y / 2.0), 11.0),
        scale: Vec3::new(GAME_SIZE.0, ground_image_size.y, 1.0),
        ..default()
    };
    let ground_bundle = ColorMesh2dBundle {
        transform: ground_transform,
        material: materials.add(assets.ground_top.clone().into()),
        mesh: meshes.add(ground_mesh.into()).into(),
        ..default()
    };
    commands.spawn(ground_bundle)
        .insert(Name::new("Grass"));

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
    let alignment = TextAlignment::Center;
    let score_text_id = commands
        .spawn(Text2dBundle {
            text: Text::from_section("0", style.clone())
                .with_alignment(alignment),
            transform: Transform::from_translation(Vec3::new(GAME_SIZE.0 / 2.0, 300.0, 50.0)),
            ..default()
        })
        .insert(Name::new("Score Text"))
        .id();
    game_data.score_text = Some(score_text_id);

    // Make sure we're in the Ready state.
    next_state.set(GameState::Ready);
}

fn reset_bird(
    mut commands: Commands,
    app_state: Res<State<AppState>>,
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
        commands.entity(entity).insert(animation::Play);
    }
}

fn reset_pipes(
    app_state: Res<State<AppState>>,
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
        commands.entity(entity).remove::<animation::Play>();
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
    mut egui_ctx: EguiContexts,
    mut tap_events: EventWriter<TapEvent>,
) {
    // TODO: Figure out why wants_pointer_input doesn't seem to be working _at all_ now.
    let keyboard_input = !egui_ctx.ctx_mut().wants_keyboard_input() && keys.just_pressed(KeyCode::Space);
    let mouse_input = !egui_ctx.ctx_mut().wants_pointer_input() && buttons.just_pressed(MouseButton::Left);
    let touch_input = !egui_ctx.ctx_mut().wants_pointer_input() && touches.iter_just_pressed().count() > 0;
    if keyboard_input || mouse_input || touch_input {
        tap_events.send_default();
    }
}

fn check_state_transition(
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut tap_events: EventReader<TapEvent>,
    bird_q: Query<&Transform, With<Bird>>,
) {
    // Making sure we drain the events.
    if tap_events.iter().next().is_none() {
        return;
    }

    match game_state.0 {
        GameState::Ready => next_game_state.set(GameState::Playing),
        GameState::Lost => {
            // Make sure the bird has hit the ground before resetting.
            if let Ok(bird_transform) = bird_q.get_single() {
                if bird_transform.translation.y <= GROUND_OFFSET * 2.0 {
                    next_game_state.set(GameState::Ready);
                }
            }
        }
        _ => {}
    }
}

fn bird_movement(
    game_state: Res<State<GameState>>,
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
        // dbg!(event);
        if let &CollisionEvent::Started(entity1, entity2, _flags) = event {
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
    mut next_state: ResMut<NextState<GameState>>,
    mut collisions: EventReader<CollisionEvent>,
    bird_q: Query<&Transform, With<Bird>>,
    pipe_body_q: Query<(), With<PipeBody>>,
) {
    // Check if bird hit the ground.
    if let Ok(transform) = bird_q.get_single() {
        if transform.translation.y <= (GROUND_OFFSET * 2.0) + BIRD_RADIUS {
            next_state.set(GameState::Lost);
            return;
        }
    }

    // Check if bird hit a pipe.
    let bird_hit_pipe = |entity1, entity2| {
        bird_q.contains(entity1) && pipe_body_q.contains(entity2)
    };
    for event in collisions.iter() {
        if let &CollisionEvent::Started(entity1, entity2, _flags) = event {
            let hit_pipe = bird_hit_pipe(entity1, entity2) || bird_hit_pipe(entity2, entity1);
            if hit_pipe {
                next_state.set(GameState::Lost);
            }
        }
    }
}

fn camera_control(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut egui_ctx: EguiContexts,
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
