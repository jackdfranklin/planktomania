use bevy::prelude::*;
use rand::Rng;

const PLAYER_RADIUS: f32 = 15.0;
const PLAYER_MAX_SPEED: f32 = 100.0;

mod fauna;
use fauna::SpeciesType;

use crate::fauna::{get_nutrition, get_radius};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, accumulate_input)
        .add_systems(Update, random_motion)
        .add_systems(Update, (update_transforms, update_camera).chain())
        .add_systems(FixedUpdate, (update_velocities, update_positions).chain())
        .add_systems(FixedUpdate, consume_plankton)
        .run();
}

#[derive(Component)]
struct Plankton;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Predator;

#[derive(Component)]
struct Mass(f32);

#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct Position(Vec2);

#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct OldPosition(Vec2);

#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct Acceleration(Vec2);

#[derive(Component)]
struct Eatable;

#[derive(Debug, Component, Clone, PartialEq, Default, Deref, DerefMut)]
struct SwimTimer(Timer);

#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct AccumulatedInput {
    movement: Vec2,
}

fn random_vec2(length: f32, rng: &mut impl Rng) -> Vec2{
    let dir: Dir2 = rng.random();
    length * dir.as_vec2()
}


fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::rng();
    commands.spawn(Camera2d);
    for _ in 0..100 {
        let species = fauna::sample_species(&mut rng);

        let x: f32 = (rng.random::<f32>() - 0.5) * 1200.0;
        let y: f32 = (rng.random::<f32>() - 0.5) * 600.0;

        commands.spawn((
            Mesh2d(meshes.add(Circle::new(get_radius(&species)))),
            MeshMaterial2d(materials.add(fauna::get_color(&species))),
            Transform::from_xyz(
                x,
                y,
                0.0,
            ),
            Plankton,
            Eatable,
            species,
            Position(Vec2::new(x, y)),
            OldPosition(Vec2::new(x, y)),
            Velocity(random_vec2(5.0, &mut rng)),
        ));
    }
    // Spawn the player
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(PLAYER_RADIUS))),
        MeshMaterial2d(materials.add(Color::srgb(0.5, 0.5, 1.0))),
        Transform::from_xyz(
            0.0,
            0.0,
            0.0,
        ),
        Plankton,
        Player,
        Predator,
        Mass(10.0),
        Position::default(),
        OldPosition::default(),
        Velocity::default(),
        Acceleration::default(),
        AccumulatedInput::default(),
        SwimTimer(Timer::from_seconds(1.5, TimerMode::Repeating)),
    ));
}

fn update_velocities(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(&mut Velocity, &mut Acceleration)>,
) {
    let drag_coeff = 0.5;
    for (mut velocity, mut acceleration) in query.iter_mut() {
        let drag = -drag_coeff * velocity.0;
        velocity.0 += acceleration.0 * fixed_time.delta_secs(); 
        velocity.0 += drag * fixed_time.delta_secs();
        let speed = velocity.0.length();
        let direction = velocity.0.normalize_or_zero();
        velocity.0 = speed.min(PLAYER_MAX_SPEED) * direction;

        acceleration.0 = Vec2::ZERO;
    }
}

fn accumulate_input(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player: Single<(&mut AccumulatedInput, &mut Velocity, &mut SwimTimer)>,
) {
    let (mut input, mut velocity, mut swim_timer) = player.into_inner();

    swim_timer.0.tick(time.delta());

    if swim_timer.is_finished() {

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

        velocity.0 += 150.0 * input.movement.normalize_or_zero();

        swim_timer.reset();
    }
}

fn update_positions(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(&mut Position, &mut OldPosition, &Velocity)>
) {
    for (mut position, mut old_position, velocity) in query.iter_mut() {
        old_position.0 = position.0;
        position.x += velocity.0.x * fixed_time.delta_secs();
        position.y += velocity.0.y * fixed_time.delta_secs();
    }
}

fn transform_to_vec2(transform: &Transform) -> Vec2 {
    Vec2::new(transform.translation.x, transform.translation.y)
}

fn distance(a: Vec2, b: Vec2) -> f32 {
    let diff = a - b;
    (diff.x * diff.x + diff.y * diff.y).sqrt()
}

fn consume_plankton(
    mut commands: Commands,
    mut predator: Query<(&Transform, &mut Mass), With<Predator>>,
    plankton_list: Query<(Entity, &SpeciesType, &Transform), With<Eatable>>,
) {
    let (predator_pos, mut mass) = predator.single_mut().unwrap();
    let predator_pos = transform_to_vec2(predator_pos);

    for (plankton_entity, species, &plankton_pos) in plankton_list.iter() {
        let plankton_pos = transform_to_vec2(&plankton_pos);
        let distance = distance(predator_pos, plankton_pos);

        let plankton_radius = get_radius(species);

        if distance < (PLAYER_RADIUS + plankton_radius) {
            mass.0 += get_nutrition(species);
            commands.entity(plankton_entity).despawn();
        }
    }
}

fn update_transforms(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(
        &mut Transform,
        &Position,
        &OldPosition,
    )>,
) {
    for (mut transform, pos, old_pos) in query.iter_mut() {
        let prev = old_pos.0;
        let current = pos.0;
        // Fraction of time-step between fixed time-step updates
        let delta = fixed_time.overstep_fraction();
        // Linear interpolate between old and current positions
        let translation = prev.lerp(current, delta); 
        transform.translation = Vec3::new(translation.x, translation.y, 0.0);
    }
}

fn update_camera(
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
    time: Res<Time>,
) {
    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    // Applies a smooth effect to camera movement using stable interpolation
    // between the camera position and the player position on the x and y axes.
    let camera_decay_rate: f32 = 2.0;
    camera
        .translation
        .smooth_nudge(&direction, camera_decay_rate, time.delta_secs());
}

fn random_motion(
    mut query: Query<&mut Acceleration>,
) {
    let mut rng = rand::rng();
    let p = 0.005;
    for mut acceleration in query.iter_mut() {
        let q = rng.random::<f32>();
        if q < p {
            acceleration.0 = random_vec2(50.0, &mut rng);
        }
    }
}
