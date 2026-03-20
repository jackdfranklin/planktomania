use std::f32::consts::PI;
use std::time::Duration;

use bevy::{prelude::*};
use bevy::window::{Window, PrimaryWindow};

use avian2d::prelude::*;

use rand::{Rng, seq::IndexedRandom};

mod fauna;
use fauna::SpeciesType;

use crate::fauna::{get_nutrition, get_radius};

mod gameworld;
use gameworld::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default().with_length_unit(0.25))
        .insert_resource(Gravity::ZERO)
        .add_plugins(GameWorldPlugin)
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, accumulate_input)
        .add_systems(PreUpdate, swimming_system)
        .add_systems(PostUpdate, update_camera)
        .add_systems(Update, spawn_new_plankton)
        .add_systems(Update, eating_system)
        .run();
}

#[derive(Component)]
struct Plankton;

#[derive(Component)]
struct Predator;

#[derive(Component)]
struct Mouth;

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
    window_query: Single<&Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut rng = rand::rng();
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.0, 0.067, 0.133)),
            ..default()
        },
    ));

    let window = window_query.into_inner();

    let width = window.width();
    let height = window.height();
    
    let copepod_handle = asset_server.load("copepod.png");
    // Spawn the player
    commands.spawn((
        RigidBody::Dynamic,
        Collider::compound(vec![
            (Vec2::new(0.0, 20.0), Rotation::IDENTITY, Collider::capsule(40.0, 100.0)),
        ]),
        ColliderDensity(100.0),
        AngularInertia(f32::INFINITY),
        Sprite {
            image: copepod_handle,
            custom_size: Some(Vec2::new(240.0, 258.0)),
            image_mode: SpriteImageMode::Scale(SpriteScalingMode::FitCenter),
            ..default()
        },
        Mesh2d(meshes.add(Capsule2d::new(40.0, 100.0))),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, 0.0, 2.5),
        Plankton,
        Player,
        Predator,
        Mass(10.0),
        LinearDamping(2.0),
        AccumulatedInput::default(),
        SwimTimer(Timer::from_seconds(0.5, TimerMode::Repeating)),
    ))
    .with_child((
            Collider::circle(15.0),
            CollidingEntities::default(),
            ColliderDensity(100.0),
            Sensor,
            Mouth,
            Transform::from_xyz(0.0, 110.0, 10.0),
        ));
}

fn accumulate_input(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player: Single<(
        &mut AccumulatedInput,
        &mut LinearVelocity,
        &mut Transform,
        &mut SwimTimer),
        With<Player>>,
) {
    let (mut input, mut velocity, mut transform, mut swim_timer) = player.into_inner();

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

        if input.movement != Vec2::ZERO {

            transform.rotation = Quat::from_rotation_arc(Vec3::Y, input.movement.normalize_or_zero().extend(0.0));

            velocity.0 = 200.0 * input.movement.normalize_or_zero();
        }

        swim_timer.reset();
    }
}

fn swimming_system(
    time: Res<Time>,
    mut swimmer_query: Query<(&mut LinearVelocity, &mut Transform, &mut SwimTimer), Without<Player>>,
) {

    let mut rng = rand::rng();

    for (mut velocity, mut transform, mut swim_timer) in swimmer_query.iter_mut() {
        if swim_timer.is_finished() {
            // Swim in a random direction
            let impulse = random_vec2(150.0, &mut rng); 
            velocity.0 = impulse;
            transform.rotation = Quat::from_rotation_arc(Vec3::Y, impulse.normalize_or_zero().extend(0.0));
            // Reset swimming timer
            swim_timer.reset();
        } else {
            swim_timer.0.tick(time.delta());
        }
    }
}

fn transform_to_vec2(transform: &Transform) -> Vec2 {
    Vec2::new(transform.translation.x, transform.translation.y)
}

fn distance(a: Vec2, b: Vec2) -> f32 {
    let diff = a - b;
    (diff.x * diff.x + diff.y * diff.y).sqrt()
}

fn print_player_pos(query: Query<&Transform, With<Player>>){
    for transform in query.iter() {
        println!("{}", transform.rotation);
    }
}

fn update_camera(
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player: Single<&Transform, With<Player>>,
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

enum DiatomSpecies {
    Star,
    Circle,
    Rod,
}

fn diatom_collider(species: &DiatomSpecies) -> Collider {
    match species {
        DiatomSpecies::Star => Collider::circle(12.5),
        DiatomSpecies::Circle => Collider::circle(12.5),
        DiatomSpecies::Rod => Collider::rectangle(50.0, 10.0),
    }
}

fn spawn_new_plankton(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    new_tiles_query: Query<(Entity, &WorldTile), Without<ActiveTile>>,
) {
    let mut rng = rand::rng();

    // Sprite handles
    let star_handle = asset_server.load("star_diatom.png");
    let circle_handle = asset_server.load("circle_diatom.png");
    let rod_handle = asset_server.load("rod_diatom.png");
    let handles = [
        (star_handle, DiatomSpecies::Star),
        (circle_handle, DiatomSpecies::Circle),
        (rod_handle, DiatomSpecies::Rod),
    ];

    for (entity, new_tile) in new_tiles_query.iter() {
        let tile_centre = lattice_to_pos(new_tile.pos());
        for _ in 0..5 {

            let (species_handle, species) = handles.choose(&mut rng).unwrap();
            let x: f32 = (rng.random::<f32>() - 0.5) * TILE_WIDTH + tile_centre.x; 
            let y: f32 = (rng.random::<f32>() - 0.5) * TILE_WIDTH + tile_centre.y;
            let theta: f32 = rng.random::<f32>() * 2.0 * PI;
            let rotation = Quat::from_rotation_z(theta);

            commands.spawn((
                RigidBody::Dynamic,
                diatom_collider(species),
                ColliderDensity(0.01),
                LinearVelocity(random_vec2(5.0, &mut rng)),
                Sprite::from_image(species_handle.clone()),
                Transform::from_xyz(x, y, 0.0)
//                .with_scale(Vec2::splat(0.5).extend(0.0))
                .with_rotation(rotation),
                Plankton,
                Eatable,
            ));
        }

        // % chance of another copepod spawning
        if rng.random::<f32>() < 0.1 {
            let x: f32 = (rng.random::<f32>() - 0.5) * TILE_WIDTH + tile_centre.x; 
            let y: f32 = (rng.random::<f32>() - 0.5) * TILE_WIDTH + tile_centre.y;
            let theta: f32 = rng.random::<f32>() * 2.0 * PI;
            let rotation = Quat::from_rotation_z(theta);
            let speed = 50.0; // * rng.random::<f32>() + 5.0;
            let size = 1.4 - 0.8 * rng.random::<f32>();
            // Rotate the initial direction to the new direction
            let velocity = speed * (rotation * Vec3::Y);

            let copepod_handle = asset_server.load("copepod.png");

            commands.spawn((
                RigidBody::Dynamic,
                Collider::compound(vec![
                    (Vec2::new(0.0, 20.0 * size), Rotation::IDENTITY, Collider::capsule(40.0 * size, 100.0 * size)),
                ]),
                ColliderDensity(100.0),
                AngularInertia(f32::INFINITY),
                LinearVelocity::from(Vec2::new(velocity.x, velocity.y)),
                Sprite {
                    image: copepod_handle,
                    custom_size: Some(size * Vec2::new(240.0, 258.0)),
                    image_mode: SpriteImageMode::Scale(SpriteScalingMode::FitCenter),
                    ..default()
                },
                Transform::from_xyz(x, y, 2.5)
                .with_rotation(rotation),
                Plankton,
                Predator,
                LinearDamping(2.0),
                SwimTimer(
                    Timer::from_seconds(1.5, TimerMode::Once)
                    .tick(Duration::from_secs_f32(1.5 * rng.random::<f32>()))
                        .clone()
                ),
            ))
            .with_child((
                Collider::circle(15.0 * size),
                CollidingEntities::default(),
                ColliderDensity(100.0),
                Sensor,
                Mouth,
                Transform::from_xyz(0.0, 110.0 * size, 2.5),
            ));
        }
        commands.entity(entity).insert(ActiveTile);
    }

}

fn eating_system(
    mut commands: Commands,
    query: Query<(Entity, &CollidingEntities), With<Mouth>>,
) {
    for (eater_entity, colliding_entities) in query.iter() {
        for entity in colliding_entities.iter() {
            commands.entity(*entity).despawn();
        }
    }
}
