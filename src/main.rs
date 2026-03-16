use bevy::{
    prelude::*,
    window::{Window, PrimaryWindow},
};
use rand::Rng;

const PLAYER_RADIUS: f32 = 15.0;
const PLAYER_MAX_SPEED: f32 = 100.0;

mod fauna;
use fauna::SpeciesType;

use crate::fauna::{get_nutrition, get_radius};

mod gameworld;
use gameworld::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GameWorldPlugin)
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, accumulate_input)
        .add_systems(Update, random_motion)
        .add_systems(Update, gamepad_input)
        .add_systems(Update, (update_transforms, update_camera).chain())
        .add_systems(Update, spawn_new_plankton)
        .add_systems(FixedUpdate, (update_velocities, update_positions).chain())
        .add_systems(FixedUpdate, consume_plankton)
        .run();
}

#[derive(Component)]
struct Plankton;

#[derive(Component)]
struct Predator;

#[derive(Component)]
struct Mass(f32);

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
    commands.spawn(Camera2d);

    let window = window_query.into_inner();

    let width = window.width();
    let height = window.height();
    for _ in 0..10 {
        let species = fauna::sample_species(&mut rng);

        let x: f32 = (rng.random::<f32>() - 0.5) * 2.0 * width;
        let y: f32 = (rng.random::<f32>() - 0.5) * 2.0 * height;

        let star_handle = asset_server.load("star_diatom_low_res.png");
        commands.spawn((
            Sprite::from_image(star_handle),
            Transform::from_xyz(
                x,
                y,
                0.0,
            ).with_scale(Vec2::splat(0.1).extend(0.0)),
            Plankton,
            Eatable,
            species,
            MovementState{position: Vec2::new(x, y), rotation: Quat::default()},
            OldMovementState{position: Vec2::new(x, y), rotation: Quat::default()},
            Velocity(random_vec2(5.0, &mut rng)),
            Acceleration::default(),
        ));
    }
    let copepod_handle = asset_server.load("copepod_low_res.png");
    // Spawn the player
    commands.spawn((
        Sprite::from_image(copepod_handle),
        Transform::from_xyz(0.0, 0.0, 2.5),
        Plankton,
        Player,
        Predator,
        Mass(10.0),
        MovementState{position: Vec2::ZERO, rotation: Quat::default()},
        OldMovementState{position: Vec2::ZERO, rotation: Quat::default()},
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

fn gamepad_usage_system(gamepads: Query<(&Name, &Gamepad)>) {
    for (name, gamepad) in &gamepads {
        println!("{name}");

        if gamepad.just_pressed(GamepadButton::North) {
            println!("{} just pressed North", name)
        }

        if let Some(left_stick_x) = gamepad.get(GamepadAxis::LeftStickX)  {
            println!("left stick X: {}", left_stick_x)
        }
        if let Some(left_stick_y) = gamepad.get(GamepadAxis::LeftStickY)  {
            println!("right stick Y: {}", left_stick_y)
        }
    }
}

fn flush_drift(input: f32) -> f32 {
    if input.abs() < 0.01 {
        0.0
    } else {
        input
    }
}

fn gamepad_input(
    gamepads: Query<&Gamepad>,
    player: Single<(
        &mut MovementState,
        &mut Velocity, 
        &Transform), 
    With<Player>>,
) {
    let (mut state, mut velocity, transform) = player.into_inner();

    for gamepad in &gamepads {
        let left_stick_x = flush_drift(gamepad.get(GamepadAxis::LeftStickX).unwrap());
        let left_stick_y = flush_drift(gamepad.get(GamepadAxis::LeftStickY).unwrap());

        let raw_direction = Vec2::new(left_stick_x, left_stick_y);
        let direction = if raw_direction.length() > 0.01 {
            raw_direction
        } else {
            Vec2::ZERO
        };

        if direction != Vec2::ZERO {

            state.rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize_or_zero().extend(0.0));

        }

        if gamepad.pressed(GamepadButton::South) {
            let movement_direction = transform.rotation * Vec3::Y;
            velocity.0 += 150.0 * movement_direction.xy();
        }


    }
}

fn accumulate_input(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player: Single<(
        &mut AccumulatedInput,
        &mut Velocity,
        &mut MovementState,
        &mut SwimTimer)>,
) {
    let (mut input, mut velocity, mut state, mut swim_timer) = player.into_inner();

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

            state.rotation = Quat::from_rotation_arc(Vec3::Y, input.movement.normalize_or_zero().extend(0.0));

            velocity.0 += 150.0 * input.movement.normalize_or_zero();
        }

        swim_timer.reset();
    }
}

fn update_positions(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(&mut MovementState, &mut OldMovementState, &Velocity)>
) {
    for (mut state, mut old_state, velocity) in query.iter_mut() {
        old_state.position = state.position;
        state.position.x += velocity.0.x * fixed_time.delta_secs();
        state.position.y += velocity.0.y * fixed_time.delta_secs();
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

//fn cull_plankton(
//    mut commands: Commands,
//    centre_tile: Res<CentreTile>,
//    plankton_query: Query<(Entity, &Position), With<Plankton>>,
//) {
//    for (plankton_entity, plankton_pos) in plankton_query {
//        if centre_tile.distance(pos_to_lattice(plankton_pos.0)) > 10 {
//            commands.entity(plankton_entity).despawn();
//        }
//    }
//}

fn update_transforms(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(
        &mut Transform,
        &MovementState,
        &OldMovementState,
    )>,
) {
    for (mut transform, state, old_state) in query.iter_mut() {
        let delta = fixed_time.overstep_fraction();
        // Linear interpolate between old and current positions
        let translation = old_state.position.lerp(state.position, delta); 
        let z_level = transform.translation.z;
        transform.translation = translation.extend(z_level);

        // Update rotation instantly
        transform.rotation = state.rotation;
    }
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

fn spawn_new_plankton(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    new_tiles_query: Query<(Entity, &WorldTile), Without<ActiveTile>>,
) {
    let mut rng = rand::rng();
    let star_handle = asset_server.load("star_diatom_low_res.png");
    for (entity, new_tile) in new_tiles_query.iter() {
        let tile_centre = lattice_to_pos(new_tile.pos());
        for _ in 0..20 {
            let species = fauna::sample_species(&mut rng);

            let x: f32 = (rng.random::<f32>() - 0.5) * TILE_WIDTH + tile_centre.x; 
            let y: f32 = (rng.random::<f32>() - 0.5) * TILE_WIDTH + tile_centre.y;

            commands.spawn((
                Sprite::from_image(star_handle.clone()),
                Transform::from_xyz(
                    x,
                    y,
                    0.0,
                ).with_scale(Vec2::splat(0.1).extend(0.0)),
                Plankton,
                Eatable,
                species,
                MovementState{position: Vec2::new(x, y), rotation: Quat::default()},
                OldMovementState{position: Vec2::new(x, y), rotation: Quat::default()},
                Velocity(random_vec2(5.0, &mut rng)),
            ));
        }
        commands.entity(entity).insert(ActiveTile);
    }
}
