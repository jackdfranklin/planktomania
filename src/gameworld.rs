use bevy::{color::palettes::css::*, prelude::*};

const RENDER_DISTANCE: u32 = 20;

const TILE_WIDTH: f32 = 250.0;

pub struct GameWorldPlugin;

impl Plugin for GameWorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CentreTile(IVec2::ZERO))
            .add_systems(Startup, spawn_starting_tiles)
            .add_systems(Update, cull_tiles)
            .add_systems(Update, spawn_new_tiles);
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

#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, Default, Deref, DerefMut)]
struct WorldTile(IVec2);

#[derive(Component)]
struct WorldTileStatus(bool);

#[derive(Resource)]
struct TileMap(HashMap<WorldTile, Entity>);

#[derive(Resource)]
struct CentreTile(IVec2);

fn spawn_starting_tiles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for i in -10..10 {
        for j in -10..10 {
            let x = (i as f32) * TILE_WIDTH;
            let y = (j as f32) * TILE_WIDTH;

            let colour = IVec2::new(i, j).chebyshev_distance(IVec2::ZERO) as f32 / (RENDER_DISTANCE as f32);

            commands.spawn((
                WorldTile(pos_to_lattice(Vec2::new(x, y))),
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

fn loaded_tiles(lattice_pos: IVec2) -> Vec<IVec2> {
    let mut tiles = Vec::new();
    for i in -10..10 {
        for j in -10..10 {
            tiles.push(IVec2{
                x: lattice_pos.x + i,
                y: lattice_pos.y + j,
            });
        }
    }
    tiles
}

fn lattice_to_pos(
    lattice_pos: IVec2,
) -> Vec2 {
    Vec2{
        x: lattice_pos.x as f32 * TILE_WIDTH,
        y: lattice_pos.y as f32 * TILE_WIDTH,
    }
}

fn pos_to_lattice(
    pos: Vec2,
) -> IVec2 {
    IVec2{
        x: (pos.x / TILE_WIDTH).round() as i32,
        y: (pos.y / TILE_WIDTH).round() as i32,
    }
}

fn spawn_new_tiles(
    mut commands: Commands,
    mut centre_tile: ResMut<CentreTile>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    pos_query: Single<&Position, With<Player>>,
) {
    let current_pos = pos_query.into_inner();
    let current_lattice_pos = pos_to_lattice(current_pos.0);

    if current_lattice_pos != centre_tile.0 {
        let current_lattice_trans = lattice_to_pos(current_lattice_pos);
//        commands.spawn((
//            WorldTile(current_lattice_pos),
//            Mesh2d(meshes.add(Rectangle::new(TILE_WIDTH, TILE_WIDTH))),
//            MeshMaterial2d(materials.add(Color::srgb(0.0, 0.0, 0.5))),
//            Transform::from_xyz(
//                current_lattice_trans.x,
//                current_lattice_trans.y,
//                0.0
//            ),
//        ));
        // Dumb way: Create a Vec of positions for new and old tiles, and find the
        // new tiles that don't exist in old
        let new_tiles = loaded_tiles(current_lattice_pos);
        let old_tiles = loaded_tiles(centre_tile.0);

        for nt in &new_tiles {
            let mut loaded = false;
            for ot in &old_tiles {
                if nt == ot {
                    loaded = true;
                }
            }
            if !loaded {
                let new_pos = lattice_to_pos(*nt);
                let colour = nt.chebyshev_distance(current_lattice_pos) as f32 / (RENDER_DISTANCE as f32);
                commands.spawn((
                    WorldTile(*nt),
                    Mesh2d(meshes.add(Rectangle::new(TILE_WIDTH, TILE_WIDTH))),
                    MeshMaterial2d(materials.add(Color::srgb(0.0, colour, 0.0))),
                    Transform::from_xyz(
                        new_pos.x,
                        new_pos.y,
                        0.0
                    ),
                ));
            }
        }
    }
    centre_tile.0 = current_lattice_pos;
    
}

fn cull_tiles(
    mut commands: Commands,
    pos_query: Single<&Position, With<Player>>,
    tile_query: Query<(Entity, &WorldTile)>,
) {
    let current_pos = pos_query.into_inner();
    let current_tile = pos_to_lattice(current_pos.0);

    for (entity, world_tile) in tile_query.iter() {
        if world_tile.0.chebyshev_distance(current_tile) > RENDER_DISTANCE {
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
