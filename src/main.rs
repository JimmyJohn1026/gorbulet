use std::{f32::consts::E, time::Duration};

use bevy::{prelude::*, window::PrimaryWindow};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

const PLAYER_RADIUS: f32 = 16.0;
const PLAYER_COLOR: Color = Color::BLUE;
const PLAYER_ACCEL: f32 = 600.0;
const PLAYER_MAX_SPEED: f32 = 300.0;

const HIT_KNOCKBACK: f32 = 500.0;
const HIT_DECAY_RATE: f32 = -0.003;

const ENEMY_RADIUS: f32 = 14.0;
const ENEMY_ACCEL: f32 = 600.0;
const ENEMY_MIN_SPEED: f32 = 150.0;
const ENEMY_MAX_SPEED: f32 = 400.0;

const SPEED_GROWTH_RATE: f32 = 0.15;
const SPEED_MIDPOINT: f32 = 20.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<InputBindings>()
        .add_state::<AppState>()
        .add_event::<HitPlayer>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_game,
                move_player,
                move_enemy,
                wraparound,
                collision_detection,
                hit_player,
            )
                .chain()
                .run_if(in_state(AppState::Game)),
        )
        .add_systems(Update, debug_start)
        .add_systems(OnEnter(AppState::Game), setup_game)
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Menu,
    Game,
}

#[derive(Resource, Default)]
struct GameInfo {
    state: GameState,
    points: i8,
}

#[derive(Debug, Default)]
enum GameState {
    #[default]
    Running,
    Hit,
}

#[derive(Resource)]
struct InputBindings {
    up: KeyCode,
    down: KeyCode,
    left: KeyCode,
    right: KeyCode,
}

impl Default for InputBindings {
    fn default() -> Self {
        Self {
            up: KeyCode::W,
            down: KeyCode::S,
            left: KeyCode::A,
            right: KeyCode::D,
        }
    }
}

#[derive(Resource)]
struct AssetHandles {
    player_mesh: Handle<Mesh>,
    player_material: Handle<ColorMaterial>,
    enemy_mesh: Handle<Mesh>,
}

impl AssetHandles {
    fn new(mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) -> Self {
        Self {
            player_mesh: meshes.add(shape::Circle::new(PLAYER_RADIUS).into()),
            player_material: materials.add(ColorMaterial::from(PLAYER_COLOR)),
            enemy_mesh: meshes.add(shape::Circle::new(ENEMY_RADIUS).into()),
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy {
    speed: f32,
}

#[derive(Bundle)]
struct EnemyBundle {
    enemy: Enemy,
    velocity: Velocity,
    color_mesh_2d_bundle: ColorMesh2dBundle,
}

#[derive(Event, Default)]
struct HitPlayer;

impl Default for EnemyBundle {
    fn default() -> Self {
        Self {
            enemy: Enemy {
                speed: ENEMY_MIN_SPEED,
            },
            velocity: Velocity(Vec3::ZERO),
            color_mesh_2d_bundle: ColorMesh2dBundle::default(),
        }
    }
}

enum SpawnSide {
    Top,
    Bottom,
    Left,
    Right,
}

impl Distribution<SpawnSide> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> SpawnSide {
        let index: u8 = rng.gen_range(0..4);
        match index {
            0 => SpawnSide::Top,
            1 => SpawnSide::Bottom,
            2 => SpawnSide::Left,
            3 => SpawnSide::Right,
            _ => unreachable!(),
        }
    }
}

#[derive(Component)]
struct Wraparound;

#[derive(Component)]
struct Velocity(Vec3);

fn setup(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(AssetHandles::new(meshes, materials));
    commands.spawn(Camera2dBundle::default());
}

fn debug_start(
    mut next_state: ResMut<NextState<AppState>>,
    player_query: Query<&Player>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::K) && player_query.is_empty() {
        next_state.set(AppState::Game);
    }
}

fn setup_game(
    mut commands: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
    asset_handles: Res<AssetHandles>,
) {
    commands.init_resource::<GameInfo>();

    commands.spawn((
        Player,
        Wraparound,
        Velocity(Vec3::ZERO),
        ColorMesh2dBundle {
            mesh: asset_handles.player_mesh.clone().into(),
            material: asset_handles.player_material.clone().into(),
            transform: Transform::from_translation(Vec3::ZERO),
            ..default()
        },
    ));
}

fn update_game(
    mut commands: Commands,
    game_info: Res<GameInfo>,
    asset_handles: Res<AssetHandles>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let spawn_side: SpawnSide = rand::random();
    let speed_float: f32 =
        1.0 / (1.0 + E.powf(-SPEED_GROWTH_RATE * (game_info.points as f32 - SPEED_MIDPOINT)));
    let speed = speed_float * (ENEMY_MAX_SPEED - ENEMY_MIN_SPEED) + ENEMY_MIN_SPEED;

    commands.spawn(EnemyBundle {
        enemy: Enemy { speed: speed },
        color_mesh_2d_bundle: ColorMesh2dBundle {
            mesh: asset_handles.enemy_mesh.clone().into(),
            material: asset_handles.player_material.clone().into(),
            transform: Transform::from_translation(get_spawn_position(window, spawn_side)),
            ..default()
        },
        ..default()
    });
}

fn get_spawn_position(window: Query<&Window, With<PrimaryWindow>>, spawn_side: SpawnSide) -> Vec3 {
    let window = window.single();
    let vertical = window.height() / 2.0 + PLAYER_RADIUS;
    let horizontal = window.width() / 2.0 + PLAYER_RADIUS;
    let rand_float: f32 = rand::random();

    match spawn_side {
        SpawnSide::Top => Vec3::new(horizontal * (2.0 * rand_float - 1.0), vertical, 0.0),
        SpawnSide::Bottom => Vec3::new(horizontal * (2.0 * rand_float - 1.0), -vertical, 0.0),
        SpawnSide::Left => Vec3::new(-horizontal, vertical * (2.0 * rand_float - 1.0), 0.0),
        SpawnSide::Right => Vec3::new(horizontal, vertical * (2.0 * rand_float - 1.0), 0.0),
    }
}

fn hit_player(
    hit_event: EventReader<HitPlayer>,
    player_transform: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemy_query: Query<(&Transform, &mut Velocity), (With<Enemy>, Without<Player>)>,
) {
    if hit_event.is_empty() {
        return;
    }

    let player_transform = player_transform.single();

    enemy_query
        .par_iter_mut()
        .for_each(|(transform, mut velocity)| {
            let direction =
                (transform.translation - player_transform.translation).normalize_or_zero();
            let distance = transform.translation.distance(player_transform.translation);

            let speed = HIT_KNOCKBACK * E.powf(HIT_DECAY_RATE * (distance - (PLAYER_RADIUS + ENEMY_RADIUS)));

            velocity.0 += direction * speed;
        });
}

fn move_player(
    bindings: Res<InputBindings>,
    input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
    time: Res<Time>,
) {
    if query.is_empty() {
        return;
    }

    let (mut transform, mut velocity) = query.single_mut();

    velocity.0 = vec3_move_toward(
        velocity.0,
        get_direction(bindings, input) * PLAYER_MAX_SPEED,
        PLAYER_ACCEL * time.delta_seconds(),
    );

    transform.translation += velocity.0 * time.delta_seconds();
}

fn get_direction(bindings: Res<InputBindings>, input: Res<Input<KeyCode>>) -> Vec3 {
    let mut direction = Vec3::ZERO;

    if input.pressed(bindings.up) || input.pressed(KeyCode::Up) {
        direction.y += 1.0;
    }
    if input.pressed(bindings.down) || input.pressed(KeyCode::Down) {
        direction.y -= 1.0;
    }
    if input.pressed(bindings.left) || input.pressed(KeyCode::Left) {
        direction.x -= 1.0;
    }
    if input.pressed(bindings.right) || input.pressed(KeyCode::Right) {
        direction.x += 1.0;
    }

    return direction.normalize_or_zero();
}

fn move_enemy(
    mut query: Query<(&mut Transform, &mut Velocity, &Enemy)>,
    player_transform: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    if query.is_empty() || player_transform.is_empty() {
        return;
    }

    let player_transform = player_transform.single();

    query
        .par_iter_mut()
        .for_each(|(mut transform, mut velocity, enemy)| {
            let direction =
                (player_transform.translation - transform.translation).normalize_or_zero();

            velocity.0 = vec3_move_toward(
                velocity.0,
                direction * enemy.speed,
                ENEMY_ACCEL * time.delta_seconds(),
            );

            transform.translation += velocity.0 * time.delta_seconds();
        });
}

fn vec3_move_toward(from: Vec3, to: Vec3, distance: f32) -> Vec3 {
    if from.distance_squared(to) <= distance.powf(2.0) {
        return to;
    }

    if let Some(direction) = (to - from).try_normalize() {
        from + direction * distance
    } else {
        from
    }
}

fn wraparound(
    mut transform_query: Query<&mut Transform, With<Wraparound>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window.single();
    transform_query.par_iter_mut().for_each(|mut transform| {
        let left = -window.width() / 2.0 - PLAYER_RADIUS;
        let right = -left;
        let top = window.height() / 2.0 + PLAYER_RADIUS;
        let bottom = -top;

        if transform.translation.x < left {
            transform.translation.x = right;
        } else if transform.translation.x > right {
            transform.translation.x = left;
        }
        if transform.translation.y > top {
            transform.translation.y = bottom;
        } else if transform.translation.y < bottom {
            transform.translation.y = top;
        }
    });
}

fn collision_detection(
    player_transform: Query<&Transform, (With<Player>, Without<Enemy>)>,
    enemy_query: Query<&Transform, (With<Enemy>, Without<Player>)>,
    mut hit_event: EventWriter<HitPlayer>,
) {
    if player_transform.is_empty() || enemy_query.is_empty() {
        return;
    }

    let player_transform = player_transform.single();
    let mut hit_player = false;

    for enemy_transform in enemy_query.iter() {
        let distance_squared = player_transform
            .translation
            .distance_squared(enemy_transform.translation);

        if distance_squared < (PLAYER_RADIUS + ENEMY_RADIUS).powf(2.0) {
            hit_player = true;
            break;
        }
    }

    if hit_player {
        hit_event.send_default();
    }
}
