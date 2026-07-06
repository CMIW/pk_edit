use crate::common::types::{ComputedStats, Gender, Move, Pokerus, StatBlock, TrainerID};
use crate::error::PokemonError;

pub trait Pokemon {
    fn level(&self) -> u8;
    fn set_level(&mut self, level: u8) -> Result<(), PokemonError>;
    fn nature(&self) -> String;
    fn set_nature(&mut self, nature: &str) -> Result<(), PokemonError>;
    fn gender(&self) -> Gender;
    fn set_gender(&mut self, gender: Gender) -> Result<(), PokemonError>;
    fn species(&self) -> String;
    fn set_species(&mut self, species: &str) -> Result<(), PokemonError>;
    fn ot_name(&self) -> String;
    fn set_ot_name(&mut self, ot_name: &str) -> Result<(), PokemonError>;
    fn ot_id(&self) -> TrainerID;
    fn set_ot_id(&mut self, ot_id: &TrainerID) -> Result<(), PokemonError>;
    fn nickname(&self) -> String;
    fn set_nickname(&mut self, nickname: &str) -> Result<(), PokemonError>;
    fn held_item(&self) -> Option<String>;
    fn set_held_item(&mut self, item: &str) -> Result<(), PokemonError>;
    fn moves(&self) -> Vec<Move>;
    fn set_move(&mut self, slot: usize, attack: &str) -> Result<(), PokemonError>;
    fn ability(&self) -> String;
    fn set_ability(&mut self, ability: &str) -> Result<(), PokemonError>;
    fn friendship(&self) -> u8;
    fn set_friendship(&mut self, value: u8) -> Result<(), PokemonError>;
    fn pokerus_status(&self) -> Pokerus;
    fn set_pokerus_status(&mut self, status: Pokerus) -> Result<(), PokemonError>;
    fn ivs(&self) -> StatBlock;
    fn set_ivs(&mut self, ivs: StatBlock) -> Result<(), PokemonError>;
    fn evs(&self) -> StatBlock;
    fn set_evs(&mut self, evs: StatBlock) -> Result<(), PokemonError>;
    fn exp(&self) -> u32;
    fn is_egg(&self) -> bool;
    fn is_shiny(&self) -> bool;
    fn is_bad_egg(&self) -> bool;
    fn to_bytes(&self) -> Vec<u8>;
    /// Returns `true` if this slot holds no Pokémon (i.e. is unoccupied).
    fn is_empty(&self) -> bool;
    /// Returns the National Pokédex number.
    fn nat_dex_number(&self) -> u16;
    /// Returns the raw personality value (PID).
    fn personality_value(&self) -> u32;
    /// Returns the Pokémon's language as a display string.
    fn language(&self) -> String;
    /// Returns the primary and optional secondary type names, or `None` if the slot is empty.
    fn typing(&self) -> Option<(String, Option<String>)>;
    /// Returns the item-ID of the Pokéball this Pokémon was caught in.
    fn pokeball_caught(&self) -> usize;
    /// Sets the Pokéball caught ID.
    ///
    /// # Errors
    /// Returns an error if `ball_id` is out of range for this generation.
    fn set_pokeball_caught(&mut self, ball_id: u8) -> Result<(), PokemonError>;
    /// Infects this Pokémon with a Pokérus strain.
    fn infect_pokerus(&mut self);
    /// Marks this Pokémon as cured of Pokérus (keeps strain, clears remaining days).
    fn cure_pokerus(&mut self);
    /// Removes Pokérus entirely, as if the Pokémon was never infected.
    fn remove_pokerus(&mut self);
    /// Returns the computed in-battle stats at the Pokémon's current level.
    fn computed_stats(&self) -> ComputedStats;
    /// Updates a single IV by name (`"HP"`, `"Attack"`, `"Defense"`, `"Sp. Atk"`, `"Sp. Def"`, `"Speed"`).
    fn update_iv(&mut self, stat: &str, value: u16);
    /// Updates a single EV by name, clamping to the per-stat and total EV limits.
    fn update_ev(&mut self, stat: &str, value: u16);
    /// Recalculates and writes the internal checksum before serialising to save bytes.
    fn update_checksum(&mut self);
}
