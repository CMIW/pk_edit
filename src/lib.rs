//! Pokémon Generation III save file editor.
//!
//! Provides utilities to read, modify, and write save files from Pokémon games such as Ruby, Sapphire, Emerald, FireRed, and LeafGreen.
//!
//! # Examples
//! ```rust no_run
//! use std::fs::File;
//! use std::io::Read;
//! use std::error::Error;
//! use std::io::BufReader;
//! use pk_edit::{SaveFile, Pokemon};
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let mut buffer = Vec::new();
//!     let file = File::open("~/Pokemon - Emerald Version/Pokemon - Emerald Version (U).sav")?;
//!     let mut buf_reader = BufReader::new(file);
//!     buf_reader.read_to_end(&mut buffer)?;
//!
//!     let save_file: SaveFile = SaveFile::new(&buffer);
//!
//!     let party = save_file.get_party();
//!
//!     let box1 = save_file.pc_box(0);
//!
//!     Ok(())
//! }
//! ```
//!
pub mod data_structure;
#[doc(hidden)]
pub mod test;

#[doc(hidden)]
pub use crate::data_structure::pokemon::Pokemon;
#[doc(hidden)]
pub use crate::data_structure::save_data::SaveFile;
#[doc(hidden)]
pub use crate::data_structure::save_data::StorageType;
