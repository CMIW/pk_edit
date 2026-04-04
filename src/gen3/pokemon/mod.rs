//! Implementation of [Pokémon data structure Gen (III)](https://bulbapedia.bulbagarden.net/wiki/Pok%C3%A9mon_data_structure_(Generation_III)).
//!
//! Pokémon data handling module.

mod crypto;
pub(crate) mod data;
mod factory;
mod pokemon;

pub use data::Stats;
pub use factory::{gen_pokemon_from_species, Gen3Factory};
pub use pokemon::Gen3Pokemon;
