use bevy::{prelude::*};

const RENDER_DISTANCE: u32 = 20;

pub const TILE_WIDTH: f32 = 250.0;

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

#[derive(Debug, Component, Clone, Copy, Default)]
pub struct MovementState{
    pub position: Vec2,
    pub rotation: f32,
}

#[derive(Debug, Component, Clone, Copy, Default)]
pub struct OldMovementState{
    pub position: Vec2,
    pub rotation: f32,
}

#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub struct Acceleration(pub Vec2);

#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, Default, Deref, DerefMut)]
pub struct WorldTile(IVec2);

pub trait Tile {
    fn new(pos: IVec2) -> Self;

    fn pos(&self) -> IVec2;

    fn distance(&self, rhs: IVec2) -> u32 {
        self.pos().manhattan_distance(rhs)
    }
}

impl Tile for WorldTile {
    fn new(pos: IVec2) -> WorldTile {
        WorldTile(pos)
    }

    fn pos(&self) -> IVec2 {
        self.0
    }

}

#[derive(Component)]
pub struct ActiveTile;

//#[derive(Resource)]
//struct TileMap(HashMap<WorldTile, Entity>);

#[derive(Debug, Resource, Clone, Copy, PartialEq, Eq, Default, Deref, DerefMut)]
pub struct CentreTile(IVec2);

impl Tile for CentreTile {

    fn new(pos: IVec2) -> CentreTile {
        CentreTile(pos)
    }

    fn pos(&self) -> IVec2 {
        self.0
    }

}

fn spawn_starting_tiles(
    mut commands: Commands,
) {
    for i in -10..10 {
        for j in -10..10 {
            let x = (i as f32) * TILE_WIDTH;
            let y = (j as f32) * TILE_WIDTH;

            commands.spawn(
                WorldTile(pos_to_lattice(Vec2::new(x, y))),
            );
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

pub fn lattice_to_pos(
    lattice_pos: IVec2,
) -> Vec2 {
    Vec2{
        x: lattice_pos.x as f32 * TILE_WIDTH,
        y: lattice_pos.y as f32 * TILE_WIDTH,
    }
}

pub fn pos_to_lattice(
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
    state_query: Single<&MovementState, With<Player>>,
) {
    let current_state = state_query.into_inner();
    let current_lattice_pos = pos_to_lattice(current_state.position);

    if current_lattice_pos != centre_tile.0 {
        // Dumb way: Create a Vec of positions for new and old tiles, and find the
        // new tiles that don't exist in old
        let new_tiles = loaded_tiles(current_lattice_pos);
        let old_tiles = loaded_tiles(centre_tile.pos());

        for nt in &new_tiles {
            let mut loaded = false;
            for ot in &old_tiles {
                if nt == ot {
                    loaded = true;
                }
            }
            if !loaded {
                commands.spawn(
                    WorldTile(*nt),
                );
            }
        }
    }
    *centre_tile = CentreTile::new(current_lattice_pos);
}

fn cull_tiles(
    mut commands: Commands,
    state_query: Single<&MovementState, With<Player>>,
    tile_query: Query<(Entity, &WorldTile)>,
) {
    let current_state = state_query.into_inner();
    let current_tile = pos_to_lattice(current_state.position);

    for (entity, world_tile) in tile_query.iter() {
        if world_tile.distance(current_tile) > RENDER_DISTANCE {
            commands.entity(entity).despawn();
        }
    }
}
