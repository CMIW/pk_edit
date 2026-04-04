//! Pokémon save file editor library.
//!
//! Entry point is [`open`], which detects the generation from raw bytes and returns an
//! [`OpenSave`] enum. Pattern-match on the variant to get the concrete save type, or use
//! [`OpenSave::game_data`] to get the matching [`AnyGameData`] dispatcher.
//!
//! # Example
//! ```rust,no_run
//! use pk_edit::OpenSave;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let data = std::fs::read("save.sav")?;
//!     let save = pk_edit::open(&data)?;
//!     let game_data = save.game_data();
//!     let species_list = game_data.species()?;
//!     match save {
//!         OpenSave::Gen3(s) => println!("Party: {}", s.get_party()?.len()),
//!     }
//!     Ok(())
//! }
//! ```

pub mod common;
pub mod error;
pub mod gen3;
pub mod misc;
pub mod traits;

#[doc(hidden)]
pub mod test;

use common::types::{Abilities, Evolution, Move, Pocket};
use error::PokemonError;
use traits::pokemon::Pokemon;
use traits::pokemon_factory::PokemonFactory;

pub use error::DetectError;
pub use error::SaveDataError;

// Convenience re-exports so GUI code can use short paths.
pub use common::types::ComputedStats;
pub use common::types::Gender;
pub use common::types::Pocket as GamePocket;
pub use common::types::Pokerus;
pub use common::types::StatBlock;
pub use common::types::TrainerID;
pub use gen3::game_data::{Gen3GameData, NATURE};
pub use gen3::pokemon::Stats as Gen3Stats;
pub use gen3::pokemon::{Gen3Factory, Gen3Pokemon};
pub use gen3::save::storage::Pocket as Gen3Pocket;
pub use gen3::save::storage::StorageType;
pub use gen3::save::SaveFile as Gen3SaveFile;
pub use traits::game_data::GameData;
pub use traits::pokemon::Pokemon as PokemonTrait;
pub use traits::pokemon_factory::PokemonFactory as PokemonFactoryTrait;

// --- AnyPokemon ---

/// A generation-dispatching wrapper around the concrete Pokémon types.
///
/// Implements [`traits::pokemon::Pokemon`] by delegating to the inner variant,
/// so widgets and editor panels that only need the common interface can accept
/// `&AnyPokemon` without caring about the generation. Gen-specific panels
/// pattern-match the variant directly.
#[derive(Debug, Clone, Copy)]
pub enum AnyPokemon {
    /// A Pokémon from a Generation III save.
    Gen3(Gen3Pokemon),
}

impl Pokemon for AnyPokemon {
    fn level(&self) -> u8 {
        match self {
            Self::Gen3(p) => p.level(),
        }
    }
    fn set_level(&mut self, level: u8) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_level(level),
        }
    }
    fn nature(&self) -> String {
        match self {
            Self::Gen3(p) => p.nature(),
        }
    }
    fn set_nature(&mut self, nature: &str) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_nature(nature),
        }
    }
    fn gender(&self) -> Gender {
        match self {
            Self::Gen3(p) => p.gender(),
        }
    }
    fn set_gender(&mut self, gender: Gender) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_gender(gender),
        }
    }
    fn species(&self) -> String {
        match self {
            Self::Gen3(p) => p.species(),
        }
    }
    fn set_species(&mut self, species: &str) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_species(species),
        }
    }
    fn ot_name(&self) -> String {
        match self {
            Self::Gen3(p) => p.ot_name(),
        }
    }
    fn set_ot_name(&mut self, ot_name: &str) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_ot_name(ot_name),
        }
    }
    fn ot_id(&self) -> TrainerID {
        match self {
            Self::Gen3(p) => p.ot_id(),
        }
    }
    fn set_ot_id(&mut self, ot_id: &TrainerID) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_ot_id(ot_id),
        }
    }
    fn nickname(&self) -> String {
        match self {
            Self::Gen3(p) => p.nickname(),
        }
    }
    fn set_nickname(&mut self, nickname: &str) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_nickname(nickname),
        }
    }
    fn held_item(&self) -> Option<String> {
        match self {
            Self::Gen3(p) => p.held_item(),
        }
    }
    fn set_held_item(&mut self, item: &str) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_held_item(item),
        }
    }
    fn moves(&self) -> Vec<Move> {
        match self {
            Self::Gen3(p) => p.moves(),
        }
    }
    fn set_move(&mut self, slot: usize, attack: &str) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_move(slot, attack),
        }
    }
    fn ability(&self) -> String {
        match self {
            Self::Gen3(p) => p.ability(),
        }
    }
    fn set_ability(&mut self, ability: &str) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_ability(ability),
        }
    }
    fn friendship(&self) -> u8 {
        match self {
            Self::Gen3(p) => p.friendship(),
        }
    }
    fn set_friendship(&mut self, value: u8) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_friendship(value),
        }
    }
    fn pokerus_status(&self) -> Pokerus {
        match self {
            Self::Gen3(p) => p.pokerus_status(),
        }
    }
    fn set_pokerus_status(&mut self, status: Pokerus) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_pokerus_status(status),
        }
    }
    fn ivs(&self) -> StatBlock {
        match self {
            Self::Gen3(p) => p.ivs(),
        }
    }
    fn set_ivs(&mut self, ivs: StatBlock) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_ivs(ivs),
        }
    }
    fn evs(&self) -> StatBlock {
        match self {
            Self::Gen3(p) => p.evs(),
        }
    }
    fn set_evs(&mut self, evs: StatBlock) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_evs(evs),
        }
    }
    fn exp(&self) -> u32 {
        match self {
            Self::Gen3(p) => p.exp(),
        }
    }
    fn is_egg(&self) -> bool {
        match self {
            Self::Gen3(p) => p.is_egg(),
        }
    }
    fn is_shiny(&self) -> bool {
        match self {
            Self::Gen3(p) => p.is_shiny(),
        }
    }
    fn is_bad_egg(&self) -> bool {
        match self {
            Self::Gen3(p) => p.is_bad_egg(),
        }
    }
    fn lowest_level(&self) -> u8 {
        match self {
            Self::Gen3(p) => p.lowest_level(),
        }
    }
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Gen3(p) => p.to_bytes(),
        }
    }
    fn is_empty(&self) -> bool {
        match self {
            Self::Gen3(p) => p.is_empty(),
        }
    }
    fn nat_dex_number(&self) -> u16 {
        match self {
            Self::Gen3(p) => p.nat_dex_number(),
        }
    }
    fn personality_value(&self) -> u32 {
        match self {
            Self::Gen3(p) => p.personality_value(),
        }
    }
    fn language(&self) -> String {
        match self {
            Self::Gen3(p) => p.language(),
        }
    }
    fn typing(&self) -> Option<(String, Option<String>)> {
        match self {
            Self::Gen3(p) => p.typing(),
        }
    }
    fn pokeball_caught(&self) -> usize {
        match self {
            Self::Gen3(p) => p.pokeball_caught(),
        }
    }
    fn set_pokeball_caught(&mut self, ball_id: u8) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_pokeball_caught(ball_id),
        }
    }
    fn infect_pokerus(&mut self) {
        match self {
            Self::Gen3(p) => p.infect_pokerus(),
        }
    }
    fn cure_pokerus(&mut self) {
        match self {
            Self::Gen3(p) => p.cure_pokerus(),
        }
    }
    fn remove_pokerus(&mut self) {
        match self {
            Self::Gen3(p) => p.remove_pokerus(),
        }
    }
    fn computed_stats(&self) -> ComputedStats {
        match self {
            Self::Gen3(p) => p.computed_stats(),
        }
    }
    fn update_iv(&mut self, stat: &str, value: u16) {
        match self {
            Self::Gen3(p) => p.update_iv(stat, value),
        }
    }
    fn update_ev(&mut self, stat: &str, value: u16) {
        match self {
            Self::Gen3(p) => p.update_ev(stat, value),
        }
    }
    fn update_checksum(&mut self) {
        match self {
            Self::Gen3(p) => p.update_checksum(),
        }
    }
}

// --- AnyGameData ---

/// A generation-dispatching wrapper around the [`GameData`] implementations.
///
/// Returned by [`OpenSave::game_data`]. Implements [`GameData`] with
/// `Box<dyn Error>` as the unified error type so the GUI does not need to
/// know which concrete implementation it is talking to.
#[derive(Debug)]
pub enum AnyGameData {
    /// Game data for a Generation III save.
    Gen3(Gen3GameData),
}

impl Default for AnyGameData {
    fn default() -> Self {
        Self::Gen3(Gen3GameData)
    }
}

impl GameData for AnyGameData {
    type Result<R> = Result<R, Box<dyn std::error::Error>>;

    fn held_items(&self) -> Self::Result<Vec<String>> {
        match self {
            Self::Gen3(g) => Ok(g.held_items()?),
        }
    }
    fn nat_dex_num(&self, species: &str) -> Self::Result<u16> {
        match self {
            Self::Gen3(g) => Ok(g.nat_dex_num(species)?),
        }
    }
    fn growth_rate(&self, dex_num: u16) -> Self::Result<String> {
        match self {
            Self::Gen3(g) => Ok(g.growth_rate(dex_num)?),
        }
    }
    fn pk_species(&self, dex_num: u16) -> Self::Result<String> {
        match self {
            Self::Gen3(g) => Ok(g.pk_species(dex_num)?),
        }
    }
    fn move_data(&self, id: usize) -> Self::Result<(String, String, u8)> {
        match self {
            Self::Gen3(g) => Ok(g.move_data(id)?),
        }
    }
    fn typing(&self, dex_num: u16) -> Self::Result<(String, Option<String>)> {
        match self {
            Self::Gen3(g) => Ok(g.typing(dex_num)?),
        }
    }
    fn gender_ratio(&self, dex_num: u16) -> Self::Result<String> {
        match self {
            Self::Gen3(g) => Ok(g.gender_ratio(dex_num)?),
        }
    }
    fn species(&self) -> Self::Result<Vec<String>> {
        match self {
            Self::Gen3(g) => Ok(g.species()?),
        }
    }
    fn moves(&self) -> Self::Result<Vec<String>> {
        match self {
            Self::Gen3(g) => Ok(g.moves()?),
        }
    }
    fn base_stats(&self, dex_num: &u16) -> Self::Result<(u16, u16, u16, u16, u16, u16)> {
        match self {
            Self::Gen3(g) => Ok(g.base_stats(dex_num)?),
        }
    }
    fn evolution(&self, dex_num: &u16) -> Self::Result<Evolution> {
        match self {
            Self::Gen3(g) => Ok(g.evolution(dex_num)?),
        }
    }
    fn abilities(&self, dex_num: u16) -> Self::Result<Abilities> {
        match self {
            Self::Gen3(g) => Ok(g.abilities(dex_num)?),
        }
    }
    fn items_in_pocket(&self, pocket: Pocket) -> Self::Result<Vec<String>> {
        match self {
            Self::Gen3(g) => Ok(g.items_in_pocket(pocket)?),
        }
    }
    fn balls_id(&self) -> Self::Result<Vec<u16>> {
        match self {
            Self::Gen3(g) => Ok(g.balls_id()?),
        }
    }
    fn item_id_by_name(&self, name: &str) -> Self::Result<usize> {
        match self {
            Self::Gen3(g) => Ok(g.item_id_by_name(name)?),
        }
    }
    fn item_sprite_id(&self, name: &str) -> Self::Result<usize> {
        match self {
            Self::Gen3(g) => Ok(g.item_sprite_id(name)?),
        }
    }
    fn balls_sprite_ids(&self) -> Self::Result<Vec<u16>> {
        match self {
            Self::Gen3(g) => Ok(g.balls_sprite_ids()?),
        }
    }
}

// --- AnyFactory ---

/// A generation-dispatching wrapper around the [`PokemonFactory`] implementations.
///
/// Returned by [`OpenSave::pokemon_factory`]. Implements [`PokemonFactory`] with
/// [`AnyPokemon`] as the output type so the GUI never needs to name a concrete
/// generation type when creating new Pokémon.
#[derive(Debug, Clone, Copy)]
pub enum AnyFactory {
    /// Factory for Generation III Pokémon.
    Gen3(Gen3Factory),
}

impl Default for AnyFactory {
    fn default() -> Self {
        Self::Gen3(Gen3Factory)
    }
}

impl PokemonFactory for AnyFactory {
    type Output = AnyPokemon;

    fn gen_pokemon_from_species(
        &self,
        species: &str,
        ot_name: &str,
        ot_id: TrainerID,
    ) -> Result<AnyPokemon, error::PokemonError> {
        match self {
            Self::Gen3(f) => f
                .gen_pokemon_from_species(species, ot_name, ot_id)
                .map(AnyPokemon::Gen3),
        }
    }
}

// --- OpenSave ---

/// The result of opening and detecting a Pokémon save file.
///
/// Each variant holds the concrete save type for that generation. Pattern-match
/// to access generation-specific methods that are not part of the common traits.
/// Use [`OpenSave::game_data`] to get the matching data provider.
#[derive(Debug)]
pub enum OpenSave {
    /// A Generation III save (Ruby, Sapphire, Emerald, FireRed, LeafGreen).
    Gen3(gen3::save::SaveFile),
}

impl OpenSave {
    /// Returns the [`AnyGameData`] dispatcher that matches this save's generation.
    pub fn game_data(&self) -> AnyGameData {
        match self {
            Self::Gen3(_) => AnyGameData::Gen3(Gen3GameData),
        }
    }

    /// Returns the [`AnyFactory`] for creating new Pokémon in this save's generation.
    pub fn pokemon_factory(&self) -> AnyFactory {
        match self {
            Self::Gen3(_) => AnyFactory::Gen3(Gen3Factory),
        }
    }

    /// Returns the party Pokémon as [`AnyPokemon`] values.
    ///
    /// # Errors
    /// Returns an error if the party section cannot be parsed.
    pub fn party(&self) -> Result<Vec<AnyPokemon>, Box<dyn std::error::Error>> {
        match self {
            Self::Gen3(s) => Ok(s.get_party()?.into_iter().map(AnyPokemon::Gen3).collect()),
        }
    }

    /// Returns the Pokémon in a PC box as [`AnyPokemon`] values.
    ///
    /// # Errors
    /// Returns an error if the box data cannot be parsed.
    pub fn pc_box(&self, number: usize) -> Result<Vec<AnyPokemon>, Box<dyn std::error::Error>> {
        match self {
            Self::Gen3(s) => Ok(s
                .pc_box(number)?
                .into_iter()
                .map(AnyPokemon::Gen3)
                .collect()),
        }
    }

    /// Returns `true` if the save file buffer is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Gen3(s) => s.is_empty(),
        }
    }

    /// Returns `true` if the PC buffer contains no Pokémon.
    pub fn is_pc_empty(&self) -> bool {
        match self {
            Self::Gen3(s) => s.is_pc_empty(),
        }
    }

    /// Returns a copy of the raw save file bytes.
    pub fn raw_data(&self) -> Vec<u8> {
        match self {
            Self::Gen3(s) => s.raw_data(),
        }
    }

    /// Saves a single Pokémon back to its storage location.
    ///
    /// # Errors
    /// Returns an error if the storage location is invalid or the Pokémon's generation
    /// does not match this save.
    pub fn save_pokemon(
        &mut self,
        storage: gen3::save::storage::StorageType,
        pokemon: AnyPokemon,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match (self, pokemon) {
            (Self::Gen3(s), AnyPokemon::Gen3(p)) => Ok(s.save_pokemon(storage, p)?),
        }
    }

    /// Swaps two Pokémon between storage locations.
    ///
    /// # Errors
    /// Returns an error if either storage location is invalid or the Pokémon's generation
    /// does not match this save.
    pub fn swap_pokemon(
        &mut self,
        from: AnyPokemon,
        from_storage: gen3::save::storage::StorageType,
        to: AnyPokemon,
        to_storage: gen3::save::storage::StorageType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match (self, from, to) {
            (Self::Gen3(s), AnyPokemon::Gen3(f), AnyPokemon::Gen3(t)) => {
                Ok(s.swap_pokemon(f, from_storage, t, to_storage)?)
            }
        }
    }

    /// Returns the trainer's name decoded from the save file.
    ///
    /// Returns an empty string if the trainer section cannot be read.
    pub fn trainer_name(&self) -> String {
        match self {
            Self::Gen3(s) => s.trainer().map(|t| t.name).unwrap_or_default(),
        }
    }

    /// Returns the trainer's ID from the save file.
    ///
    /// Returns a default [`TrainerID`] if the trainer section cannot be read.
    pub fn trainer_id(&self) -> TrainerID {
        match self {
            Self::Gen3(s) => s.trainer().map(|t| t.id).unwrap_or_default(),
        }
    }

    /// Returns the contents of the requested bag pocket.
    ///
    /// Each entry is `(item_name, quantity)`.
    ///
    /// # Errors
    /// Returns an error if the pocket data cannot be decoded.
    pub fn pocket(
        &self,
        pocket: gen3::save::storage::Pocket,
    ) -> Result<Vec<(String, u16)>, error::SaveDataError> {
        match self {
            Self::Gen3(s) => s.pocket(pocket),
        }
    }

    /// Writes updated bag pocket data back into the save file.
    ///
    /// # Errors
    /// Returns an error if the pocket data cannot be encoded or the section is missing.
    pub fn save_pocket(
        &mut self,
        pocket_type: gen3::save::storage::Pocket,
        pocket_list: Vec<(String, u16)>,
    ) -> Result<(), error::SaveDataError> {
        match self {
            Self::Gen3(s) => s.save_pocket(pocket_type, pocket_list),
        }
    }
}

/// Opens a raw save file buffer, detects the generation, and returns an [`OpenSave`].
///
/// Detection is based on file size; magic-byte checks will be added when Gen IV
/// and BDSP support is implemented.
///
/// # Errors
/// Returns [`DetectError::TooSmall`] if the buffer is under 128 bytes.
/// Returns [`DetectError::UnknownFormat`] if no supported format matches.
pub fn open(data: &[u8]) -> Result<OpenSave, DetectError> {
    const GEN3_SIZE: usize = 131_072; // 128 KB

    if data.len() < 128 {
        return Err(DetectError::TooSmall(data.len()));
    }

    match data.len() {
        GEN3_SIZE => Ok(OpenSave::Gen3(gen3::save::SaveFile::new(data))),
        n => Err(DetectError::UnknownFormat(n)),
    }
}
