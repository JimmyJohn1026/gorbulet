use bevy::{prelude::*, window::PrimaryWindow};

const PLAYER_RADIUS: f32 = 16.0;
const PLAYER_COLOR: Color = Color::BLUE;
const PLAYER_ACCEL: f32 = 600.0;
const PLAYER_MAX_SPEED: f32 = 300.0;

const ENEMY_RADIUS: f32 = 14.0;
const ENEMY_ACCEL: f32 = 600.0;
const ENEMY_MIN_SPEED: f32 = 150.0;
const ENEMY_MAX_SPEED: f32 = 400.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<InputBindings>()
        .add_systems(Startup, setup)
        .add_systems(Update, (move_player, move_enemy, wraparound).chain())
        .add_systems(Update, debug_start)
        .run();
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
    player: (Handle<Mesh>, Handle<ColorMaterial>),
    enemy: Handle<Mesh>,
}

impl AssetHandles {
    fn new(mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) -> Self {
        Self {
            player: (
                meshes.add(shape::Circle::new(PLAYER_RADIUS).into()),
                materials.add(ColorMaterial::from(PLAYER_COLOR)),
            ),
            enemy: meshes.add(shape::Circle::new(ENEMY_RADIUS).into()),
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy {
    speed: f32,
}

#[derive(Component)]
struct Wraparound;

#[derive(Component)]
struct Velocity(Vec3);

enum SpawnSide {
    Top,
    Bottom,
    Left,
    Right,
}

fn setup(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(AssetHandles::new(meshes, materials));
    commands.spawn(Camera2dBundle::default());
}

fn debug_start(
    player_query: Query<&Player>,
    commands: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
    asset_handles: Res<AssetHandles>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::K) && player_query.is_empty() {
        spawn_player(player_query, commands, asset_handles);
    } else if input.just_pressed(KeyCode::L) {
        spawn_enemy(commands, asset_handles, window, 0.5, SpawnSide::Top);
    }
}

fn spawn_player(
    player_query: Query<&Player>,
    mut commands: Commands,
    asset_handles: Res<AssetHandles>,
) {
    if !player_query.is_empty() {
        panic!("Tried to spawn the player when one was already present");
    }

    commands.spawn((
        Player,
        Wraparound,
        Velocity(Vec3::ZERO),
        ColorMesh2dBundle {
            mesh: asset_handles.player.0.clone().into(),
            material: asset_handles.player.1.clone().into(),
            transform: Transform::from_translation(Vec3::ZERO),
            ..default()
        },
    ));
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

fn spawn_enemy(
    mut commands: Commands,
    asset_handles: Res<AssetHandles>,
    window: Query<&Window, With<PrimaryWindow>>,
    speed_weight: f32,
    spawn_side: SpawnSide,
) {
    commands.spawn((
        Enemy {
            speed: speed_weight * (ENEMY_MAX_SPEED - ENEMY_MIN_SPEED) + ENEMY_MIN_SPEED,
        },
        Velocity(Vec3::ZERO),
        ColorMesh2dBundle {
            mesh: asset_handles.enemy.clone().into(),
            material: asset_handles.player.1.clone().into(),
            //material: materials.add(ColorMaterial::from(PLAYER_COLOR)),
            transform: Transform::from_translation(get_spawn_position(window, spawn_side)),
            ..default()
        },
    ));
}

fn get_spawn_position(window: Query<&Window, With<PrimaryWindow>>, spawn_side: SpawnSide) -> Vec3 {
    let window = window.single();
    let vertical = window.height() / 2.0 + PLAYER_RADIUS;
    let horizontal = window.width() / 2.0 + PLAYER_RADIUS;
    let rand_float: f32 = rand::random();

    match spawn_side {
        SpawnSide::Top => Vec3::new(horizontal * (2.0 * rand_float - 1.0), vertical, 0.0),
        _ => Vec3::ZERO,
    }
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

            velocity.0 = direction * enemy.speed;

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
