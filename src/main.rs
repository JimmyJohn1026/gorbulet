use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

const PLAYER_RADIUS: f32 = 16.0;
const PLAYER_COLOR: Color = Color::BLUE;
const PLAYER_ACCEL: f32 = 600.0;
const PLAYER_MAX_SPEED: f32 = 300.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<InputBindings>()
        .add_systems(Startup, setup)
        .add_systems(Update, move_player)
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Velocity(Vec3);

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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        Player,
        Velocity(Vec3::ZERO),
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(PLAYER_RADIUS).into()).into(),
            material: materials.add(ColorMaterial::from(PLAYER_COLOR)),
            transform: Transform::from_translation(Vec3::new(-150.0, 0.0, 0.0)),
            ..default()
        },
    ));
}

fn move_player(
    bindings: Res<InputBindings>,
    input: Res<Input<KeyCode>>,
    mut transform: Query<&mut Transform, With<Player>>,
    mut velocity: Query<&mut Velocity, With<Player>>,
    time: Res<Time>,
) {
    let mut transform = transform.single_mut();
    let mut velocity = velocity.single_mut();

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

    return direction;
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
