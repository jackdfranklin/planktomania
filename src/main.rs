use bevy::prelude::*;
use rand::{Rng};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, accumulate_input)
        .add_systems(FixedUpdate, update_positions)
        .run();
}

#[derive(Component)]
struct Plankton;

#[derive(Component)]
struct Player;

#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct AccumulatedInput {
    movement: Vec2,
}


fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::rng();
    commands.spawn(Camera2d);
    for _ in 0..100 {
        commands.spawn((
            Mesh2d(meshes.add(Circle::new(15.0))),
            MeshMaterial2d(materials.add(Color::srgb(1.0, 0.5, 0.5))),
            Transform::from_xyz(
                (rng.random::<f32>() - 0.5) * 1200.0,
                (rng.random::<f32>() - 0.5) * 600.0,
                0.0
            ),
            Plankton,
        ));
    }
    // Spawn the player
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(15.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.5, 0.5, 1.0))),
        Transform::from_xyz(
            (rng.random::<f32>() - 0.5) * 1200.0,
            (rng.random::<f32>() - 0.5) * 600.0,
            0.0
        ),
        Plankton,
        Player,
        Velocity::default(),
        AccumulatedInput::default(),
    ));
}

fn accumulate_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player: Single<(&mut AccumulatedInput, &mut Velocity)>,
) {
    let (mut input, mut velocity) = player.into_inner();
    
    input.movement = Vec2::ZERO;
    if keyboard_input.pressed(KeyCode::KeyW) {
        input.movement.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        input.movement.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        input.movement.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        input.movement.x += 1.0;
    }

    velocity.0 = 100.0 * input.movement.normalize_or_zero();
}

fn update_positions(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(
        &mut Transform,
        &Velocity,
    )>,
) {

    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x += velocity.0.x * fixed_time.delta_secs();
        transform.translation.y += velocity.0.y * fixed_time.delta_secs();
    }
}
