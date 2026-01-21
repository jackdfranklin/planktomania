use bevy::{color::palettes::css::*, prelude::*};

const RENDER_DISTANCE: f32 = 1000.0;

const TILE_WIDTH: f32 = 100.0;

pub struct GameWorldPlugin;

impl Plugin for GameWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_starting_tiles)
           .add_systems(Update, cull_tiles);
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub struct Position(pub Vec2);

#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub struct OldPosition(pub Vec2);

#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub struct Acceleration(pub Vec2);

#[derive(Component)]
struct WorldTile(IVec2);

fn spawn_starting_tiles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let nx = 10;
    let ny = 10;
    for i in 0..nx {
        for j in 0..ny {
            let x = 0.5 * RENDER_DISTANCE * (-1.0 + 2.0 * (i as f32 / nx as f32));
            let y = 0.5 * RENDER_DISTANCE * (-1.0 + 2.0 * (j as f32 / nx as f32));

            let colour = Vec3::new(x, y, 0.0).length() / RENDER_DISTANCE;

            commands.spawn((
                WorldTile(Vec2::new(x, y).as_ivec2()),
                Mesh2d(meshes.add(Rectangle::new(TILE_WIDTH, TILE_WIDTH))),
                MeshMaterial2d(materials.add(Color::srgb(colour, 0.0, 0.0))),
                Transform::from_xyz(
                    x,
                    y,
                    0.0,
                ),
            ));
        }
    }
}

fn spawn_new_tiles(
    mut commands: Commands,
    pos_query: Single<(&Position, &OldPosition), With<Player>>,
) {
    let (current_pos, old_pos) = pos_query.into_inner();
    
}

fn cull_tiles(
    mut commands: Commands,
    pos_query: Single<&Position, With<Player>>,
    tile_query: Query<(Entity, &WorldTile)>,
) {
    let current_pos = pos_query.into_inner();
    let current_tile = current_pos.0.as_ivec2();

    for (entity, world_tile) in tile_query.iter() {
        if world_tile.0.chebyshev_distance(current_tile) > 1000 {
            commands.entity(entity).despawn();
        }
    }
}

fn draw_gizmos(
    mut gizmos: Gizmos
) {
    gizmos.grid_2d(
        Isometry2d::IDENTITY,
        UVec2::new(10, 10),
        Vec2::new(250., 250.),
        // Dark gray
        RED
    )
        .outer_edges();

    gizmos.grid_2d(
        Isometry2d::IDENTITY,
        UVec2::new(10, 10),
        Vec2::new(500., 500.),
        // Dark gray
        LinearRgba::gray(0.05),
    )
        .outer_edges();

}
