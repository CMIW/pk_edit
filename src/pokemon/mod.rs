//! Implementation of [Pokémon data structure Gen (III)](https://bulbapedia.bulbagarden.net/wiki/Pok%C3%A9mon_data_structure_(Generation_III)).
//!
//! Pokemon data handling module
//!
//! This module manages the save file sections and structures for handling Pokémon data in Gen III games.
//! It includes tools for accessing, decrypting, and managing the PC buffer, trainer data, and more.
//!
//! # Example Usage
//! ## Modifying a Pokémon's Attributes
//! ```rust
//! use std::fs::File;
//! use std::io::BufReader;
//! use pk_edit::SaveFile;
//! use pk_edit::StorageType;
//! use std::io::Read;
//!
//! let mut buffer = Vec::new();
//! let file = File::open("~/Pokemon - Emerald Version/Pokemon - Emerald Version (U).sav")?;
//! let mut buf_reader = BufReader::new(file);
//! buf_reader.read_to_end(&mut buffer)?;
//!
//! let save_file: SaveFile = SaveFile::new(&buffer);
//! let mut pokemon = save_file.pc_box(0)[0];
//!
//! pokemon.set_friendship(100);
//! pokemon.set_level(50);
//! save_file.save_pokemon(StorageType::PC, pokemon)?;
//! ```
//! ## Viewing Pokémon Data
//! ```rust
//! let mut buffer = Vec::new();
//! let file = File::open("~/Pokemon - Emerald Version/Pokemon - Emerald Version (U).sav")?;
//! let mut buf_reader = BufReader::new(file);
//! buf_reader.read_to_end(&mut buffer)?;
//!
//! let save_file: SaveFile = SaveFile::new(&buffer);
//! let pokemon = save_file.pc_box(0)[0];
//! println!("Level: {}, Friendship: {}", pokemon.level(), pokemon.friendship());
//! ```
mod crypto;
mod data;
mod entity;
mod factory;
mod stats;

pub use data::Evolution;
pub use data::{Gender, Pokerus};
pub use entity::Pokemon;
pub use factory::gen_pokemon_from_species;
pub use stats::Stats;
