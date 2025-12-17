use bevy::prelude::*;
use rand::{Rng};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

#[derive(Component)]
#[require(Transform)]
struct Plankton;

#[derive(Component)]
struct Player;

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
    ));
}
