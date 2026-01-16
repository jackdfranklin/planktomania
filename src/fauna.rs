use bevy::prelude::*;
use rand::Rng;

#[derive(Debug, Component)]
pub enum SpeciesType {
    RedPlankton,
    GreenPlankton,
}

pub fn get_nutrition(species: &SpeciesType) -> u32 {
    match species {
        SpeciesType::RedPlankton => 1,
        SpeciesType::GreenPlankton => 1,
    }
}

pub fn get_radius(species: &SpeciesType) -> f32 {
    match species {
        SpeciesType::RedPlankton => 15.0,
        SpeciesType::GreenPlankton => 10.0,
    }
}

pub fn get_color(species: &SpeciesType) -> Color {
    match species {
        SpeciesType::RedPlankton => Color::srgb(1.0, 0.5, 0.5),
        SpeciesType::GreenPlankton => Color::srgb(0.5, 1.0, 0.5),
    }
}

pub fn sample_species(rng: &mut impl Rng) -> SpeciesType {
    if rng.random::<f32>() < 0.5 {
        SpeciesType::RedPlankton
    } else {
        SpeciesType::GreenPlankton
    }
}
