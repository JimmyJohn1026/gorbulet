use std::f32::consts::{E, PI};

use bevy::{prelude::*, window::PrimaryWindow};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

const PLAYER_RADIUS: f32 = 16.0;
const PLAYER_COLOR: Color = Color::BLUE;
const PLAYER_HEALTH: i8 = 3;
const PLAYER_INVINCIBILITY_TIME: f32 = 2.0;
const PLAYER_ACCEL: f32 = 800.0;
const PLAYER_MAX_SPEED: f32 = 300.0;

const HIT_KNOCKBACK: f32 = 700.0;
const HIT_DECAY_RATE: f32 = -0.002;
const HIT_TRAUMA: f32 = 70.0;

const ENEMY_RADIUS: f32 = 14.0;
const ENEMY_COLOR: Color = Color::RED;
const ENEMY_MIN_ACCEL: f32 = 400.0;
const ENEMY_MAX_ACCEL: f32 = 700.0;
const ENEMY_MIN_SPEED: f32 = 200.0;
const ENEMY_MAX_SPEED: f32 = 500.0;
const ENEMY_COIN_PULL: f32 = 8.0;

const SPEED_GROWTH_RATE: f32 = 0.15;
const SPEED_MIDPOINT: f32 = 20.0;
const SPEED_MAX_DEVIATION: f32 = 50.0;

const COIN_RADIUS: f32 = 8.0;
const COIN_COLOR: Color = Color::YELLOW;

const SCREEN_SHAKE_X_FREQUENCY: f32 = 10.0;
const SCREEN_SHAKE_Y_FREQUENCY: f32 = 1.0;
const SCREEN_SHAKE_LERP: f32 = 0.15;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<InputBindings>()
        .add_state::<AppState>()
        .add_event::<HitPlayer>()
        .add_event::<HitCoin>()
        .add_systems(Startup, setup)
        .add_systems(Update, screen_shake)
        .add_systems(
            Update,
            (
                move_player,
                move_enemy,
                wraparound,
                enemy_collision,
                coin_collision,
                invincibility_timer,
                hit_player,
                hit_coin,
            )
                .chain()
                .run_if(in_state(AppState::Game)),
        )
        .add_systems(Update, debug_start)
        .add_systems(OnEnter(AppState::Game), setup_game)
        .add_systems(OnExit(AppState::Game), cleanup_game)
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    Menu,
    #[default]
    Game,
}

#[derive(Resource)]
struct GameInfo {
    points: i8,
    health: i8,
    is_player_invincible: bool,
}

impl Default for GameInfo {
    fn default() -> Self {
        Self {
            points: 0,
            health: PLAYER_HEALTH,
            is_player_invincible: false,
        }
    }
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
    font: Handle<Font>,
    player_mesh: Handle<Mesh>,
    player_material: Handle<ColorMaterial>,
    hit_sound: Handle<AudioSource>,
    enemy_mesh: Handle<Mesh>,
    enemy_material: Handle<ColorMaterial>,
    coin_mesh: Handle<Mesh>,
    coin_material: Handle<ColorMaterial>,
    coin_sound: Handle<AudioSource>,
}

impl AssetHandles {
    fn new(
        asset_server: Res<AssetServer>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        Self {
            font: asset_server.load("lato.ttf"),
            player_mesh: meshes.add(shape::Circle::new(PLAYER_RADIUS).into()),
            player_material: materials.add(ColorMaterial::from(PLAYER_COLOR)),
            hit_sound: asset_server.load("hit.ogg"),
            enemy_mesh: meshes.add(shape::Circle::new(ENEMY_RADIUS).into()),
            enemy_material: materials.add(ColorMaterial::from(ENEMY_COLOR)),
            coin_mesh: meshes.add(shape::Circle::new(COIN_RADIUS).into()),
            coin_material: materials.add(ColorMaterial::from(COIN_COLOR)),
            coin_sound: asset_server.load("coin.ogg"),
        }
    }
}

#[derive(Component, Default)]
struct ScreenShake {
    trauma: f32,
    time: f32,
}

impl ScreenShake {
    fn add_trauma(&mut self, trauma: f32) {
        self.trauma += trauma;
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct InvincibilityTimer(Timer);

#[derive(Component)]
struct Enemy {
    speed: f32,
    accel: f32,
    future_prediction: f32,
    coin_pull: f32,
}

#[derive(Bundle)]
struct EnemyBundle {
    enemy: Enemy,
    wraparound: Wraparound,
    velocity: Velocity,
    color_mesh_2d_bundle: ColorMesh2dBundle,
}

impl Default for EnemyBundle {
    fn default() -> Self {
        Self {
            enemy: Enemy {
                speed: ENEMY_MIN_SPEED,
                accel: ENEMY_MIN_ACCEL,
                future_prediction: 0.0,
                coin_pull: 0.0,
            },
            wraparound: Wraparound::default(),
            velocity: Velocity(Vec3::ZERO),
            color_mesh_2d_bundle: ColorMesh2dBundle::default(),
        }
    }
}

#[derive(Event, Default)]
struct HitPlayer;

#[derive(Component)]
struct Coin;

#[derive(Event, Default)]
struct HitCoin;

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

#[derive(Component, Default)]
struct Wraparound {
    radius: f32,
}

#[derive(Component)]
struct Velocity(Vec3);

fn setup(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(AssetHandles::new(asset_server, meshes, materials));
    commands.spawn((Camera2dBundle::default(), ScreenShake::default()));
}

fn debug_start(mut screen_shake: Query<&mut ScreenShake>, mut next_state: ResMut<NextState<AppState>>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::K) {
        next_state.set(AppState::Game);
    } else if input.just_pressed(KeyCode::L) {
        next_state.set(AppState::Menu);
    } else if input.just_pressed(KeyCode::T) {
        let mut screen_shake = screen_shake.single_mut();
        screen_shake.add_trauma(200.0);
    }
}

fn setup_game(
    mut commands: Commands,
    window: Query<&Window, With<PrimaryWindow>>,

    asset_handles: Res<AssetHandles>,
) {
    commands.init_resource::<GameInfo>();

    commands.spawn(InvincibilityTimer(Timer::from_seconds(
        PLAYER_INVINCIBILITY_TIME,
        TimerMode::Once,
    )));

    commands.spawn(Text2dBundle {
        text: Text::from_section(
            "0",
            TextStyle {
                font: asset_handles.font.clone(),
                font_size: 420.0,
                color: Color::DARK_GRAY,
            },
        )
        .with_alignment(TextAlignment::Center),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
        ..default()
    });

    commands.spawn((
        Player,
        Wraparound {
            radius: PLAYER_RADIUS,
        },
        Velocity(Vec3::ZERO),
        ColorMesh2dBundle {
            mesh: asset_handles.player_mesh.clone().into(),
            material: asset_handles.player_material.clone().into(),
            transform: Transform::from_translation(Vec3::ZERO),
            ..default()
        },
    ));

    let window = window.single();

    commands.spawn((
        Coin,
        ColorMesh2dBundle {
            mesh: asset_handles.coin_mesh.clone().into(),
            material: asset_handles.coin_material.clone(),
            transform: Transform::from_translation(get_coin_spawn_position(
                window.width(),
                window.height(),
            )),
            ..default()
        },
    ));
}

fn cleanup_game(
    mut commands: Commands,
    query: Query<
        Entity,
        (
            Without<Camera2d>,
            Without<Window>,
            Without<Handle<AudioSource>>,
            Without<PlaybackSettings>,
        ),
    >,
) {
    commands.remove_resource::<GameInfo>();

    query.iter().for_each(|entity| {
        commands.entity(entity).despawn();
    });
}

fn hit_coin(
    mut hit_event: EventReader<HitCoin>,
    mut game_info: ResMut<GameInfo>,
    mut score_text: Query<&mut Text>,
    mut commands: Commands,
    mut transform: Query<&mut Transform, With<Coin>>,
    asset_handles: Res<AssetHandles>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    if hit_event.is_empty() {
        return;
    }

    hit_event.clear();
    game_info.points += 1;

    let mut score_text = score_text.single_mut();
    score_text.sections[0].value = game_info.points.to_string();

    commands.spawn(AudioBundle {
        source: asset_handles.coin_sound.clone(),
        ..default()
    });

    let window = window.single();
    let mut transform = transform.single_mut();
    transform.translation = get_coin_spawn_position(window.width(), window.height());

    let spawn_side: SpawnSide = rand::random();

    let speed_float: f32 =
        1.0 / (1.0 + E.powf(-SPEED_GROWTH_RATE * (game_info.points as f32 - SPEED_MIDPOINT)));
    let deviation = SPEED_MAX_DEVIATION * (2.0 * rand::random::<f32>() - 1.0);
    let speed = speed_float * (ENEMY_MAX_SPEED - ENEMY_MIN_SPEED) + ENEMY_MIN_SPEED + deviation;

    let accel = rand::random::<f32>() * (ENEMY_MAX_ACCEL - ENEMY_MIN_ACCEL) + ENEMY_MIN_ACCEL;

    let future_prediction: f32 = rand::random();

    let coin_pull = 2.0 * rand::random::<f32>() - 1.0;

    commands.spawn(EnemyBundle {
        enemy: Enemy {
            speed,
            accel,
            future_prediction,
            coin_pull,
        },
        wraparound: Wraparound {
            radius: ENEMY_RADIUS,
        },
        color_mesh_2d_bundle: ColorMesh2dBundle {
            mesh: asset_handles.enemy_mesh.clone().into(),
            material: asset_handles.enemy_material.clone(),
            transform: Transform::from_translation(get_enemy_spawn_position(
                window.width(),
                window.height(),
                spawn_side,
            )),
            ..default()
        },
        ..default()
    });
}

fn get_coin_spawn_position(width: f32, height: f32) -> Vec3 {
    let x_float: f32 = rand::random();
    let y_float: f32 = rand::random();

    Vec3::new(width * (x_float - 0.5), height * (y_float - 0.5), -1.0)
}

fn get_enemy_spawn_position(width: f32, height: f32, spawn_side: SpawnSide) -> Vec3 {
    let vertical = height / 2.0 + PLAYER_RADIUS;
    let horizontal = width / 2.0 + PLAYER_RADIUS;
    let rand_float: f32 = rand::random();

    match spawn_side {
        SpawnSide::Top => Vec3::new(horizontal * (2.0 * rand_float - 1.0), vertical, 1.0),
        SpawnSide::Bottom => Vec3::new(horizontal * (2.0 * rand_float - 1.0), -vertical, 1.0),
        SpawnSide::Left => Vec3::new(-horizontal, vertical * (2.0 * rand_float - 1.0), 1.0),
        SpawnSide::Right => Vec3::new(horizontal, vertical * (2.0 * rand_float - 1.0), 1.0),
    }
}

fn hit_player(
    mut hit_event: EventReader<HitPlayer>,
    mut game_info: ResMut<GameInfo>,
    mut timer: Query<&mut InvincibilityTimer>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut screen_shake: Query<&mut ScreenShake>,
    asset_handles: Res<AssetHandles>,
    player_transform: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemy_query: Query<(&Transform, &mut Velocity), (With<Enemy>, Without<Player>)>,
) {
    if hit_event.is_empty() {
        return;
    }

    hit_event.clear();
    game_info.health -= 1;
    game_info.is_player_invincible = true;
    let mut timer = timer.single_mut();
    timer.0.reset();

    commands.spawn(AudioBundle {
        source: asset_handles.hit_sound.clone(),
        ..default()
    });

    let mut screen_shake = screen_shake.single_mut();
    screen_shake.add_trauma(HIT_TRAUMA);

    if game_info.health <= 0 {
        next_state.set(AppState::Menu);
    }

    let player_transform = player_transform.single();

    enemy_query
        .par_iter_mut()
        .for_each(|(transform, mut velocity)| {
            let direction =
                (transform.translation - player_transform.translation).normalize_or_zero();
            let distance = transform.translation.distance(player_transform.translation);

            let speed = HIT_KNOCKBACK
                * E.powf(HIT_DECAY_RATE * (distance - (PLAYER_RADIUS + ENEMY_RADIUS)));

            velocity.0 += direction * speed;
        });
}

fn invincibility_timer(
    time: Res<Time>,
    mut timer: Query<&mut InvincibilityTimer>,
    mut game_info: ResMut<GameInfo>,
) {
    let mut timer = timer.single_mut();
    if timer.0.tick(time.delta()).just_finished() {
        game_info.is_player_invincible = false;
    }
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
    player_query: Query<(&Transform, &Velocity), (With<Player>, Without<Enemy>)>,
    coin_transform: Query<&Transform, (With<Coin>, Without<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    if query.is_empty() || player_query.is_empty() {
        return;
    }

    let (player_transform, player_velocity) = player_query.single();
    let coin_transform = coin_transform.single();

    query
        .par_iter_mut()
        .for_each(|(mut transform, mut velocity, enemy)| {
            let track_position =
                player_transform.translation + player_velocity.0 * enemy.future_prediction;
            let direction = (track_position - transform.translation).normalize_or_zero();

            velocity.0 = vec3_move_toward(
                velocity.0,
                direction * enemy.speed,
                enemy.accel * time.delta_seconds(),
            );

            let coin_direction =
                (coin_transform.translation - transform.translation).normalize_or_zero();
            velocity.0 += coin_direction * enemy.coin_pull * ENEMY_COIN_PULL;

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
    mut query: Query<(&mut Transform, &Wraparound)>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window.single();
    query
        .par_iter_mut()
        .for_each(|(mut transform, wraparound)| {
            let left = -window.width() / 2.0 - wraparound.radius;
            let right = -left;
            let top = window.height() / 2.0 + wraparound.radius;
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

fn enemy_collision(
    game_info: Res<GameInfo>,
    player_transform: Query<&Transform, (With<Player>, Without<Enemy>)>,
    enemy_query: Query<&Transform, (With<Enemy>, Without<Player>)>,
    mut hit_event: EventWriter<HitPlayer>,
) {
    if game_info.is_player_invincible || player_transform.is_empty() || enemy_query.is_empty() {
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

fn coin_collision(
    player_transform: Query<&Transform, (With<Player>, Without<Coin>)>,
    coin_transform: Query<&Transform, (With<Coin>, Without<Player>)>,
    mut hit_event: EventWriter<HitCoin>,
) {
    if player_transform.is_empty() || coin_transform.is_empty() {
        return;
    }

    let player_transform = player_transform.single();
    let coin_transform = coin_transform.single();

    let distance_squared = player_transform
        .translation
        .distance_squared(coin_transform.translation);

    if distance_squared < (PLAYER_RADIUS + COIN_RADIUS).powf(2.0) {
        hit_event.send_default();
    }
}

fn screen_shake(
    mut query: Query<(&mut Transform, &mut ScreenShake), With<Camera>>,
    time: Res<Time>,
) {
    let (mut transform, mut screen_shake) = query.single_mut();

    if screen_shake.trauma <= 0.0 {
        screen_shake.time = 0.0;
        return;
    }

    screen_shake.trauma = lerp(screen_shake.trauma, 0.0, SCREEN_SHAKE_LERP);

    screen_shake.time += time.delta_seconds();

    transform.translation.x =
        screen_shake.trauma * (2.0 * PI * SCREEN_SHAKE_X_FREQUENCY * screen_shake.time).sin();
    transform.translation.y =
        screen_shake.trauma * (2.0 * PI * SCREEN_SHAKE_Y_FREQUENCY * screen_shake.time).sin();

}

fn lerp(from: f32, to: f32, float: f32) -> f32 {
    from + float * (to - from)
}
