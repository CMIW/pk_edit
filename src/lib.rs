//! Pokémon save file editor library.
//!
//! Entry point is [`open`], which detects the generation from raw bytes and returns an
//! [`OpenSave`] enum. Pattern-match on the variant to get the concrete save type, or use
//! [`OpenSave::game_data`] to get the matching [`AnyGameData`] dispatcher.
//!
//! # Example
//! ```rust,no_run
//! use pk_edit::GameData;
//! use pk_edit::OpenSave;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let data = std::fs::read("save.sav")?;
//!     let save = pk_edit::open(&data)?;
//!     let game_data = save.game_data();
//!     let species_list = game_data.species()?;
//!     match save {
//!         OpenSave::Gen3(s) => println!("Party: {}", s.get_party()?.len()),
//!         _ => {}
//!     }
//!     Ok(())
//! }
//! ```

pub mod bdsp;
pub mod common;
pub mod error;
pub mod gen3;
pub mod lumi;
pub mod misc;
pub mod traits;

#[doc(hidden)]
pub mod test;

use byteorder::{ByteOrder, LittleEndian};
pub use common::types::{Abilities, BagItem, Evolution, Move, Pocket};
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
pub use gen3::save::trainer::GameVersion;
pub use gen3::save::trainer::TimePlayed;
pub use gen3::save::trainer::Trainer;
pub use common::types::TrainerID;
pub use gen3::game_data::{Gen3GameData, NATURE};
pub use gen3::pokemon::Stats as Gen3Stats;
pub use gen3::pokemon::{Gen3Factory, Gen3Pokemon};
pub use gen3::save::storage::Pocket as Gen3Pocket;
pub use gen3::save::storage::StorageType;
pub use gen3::save::SaveFile as Gen3SaveFile;
pub use bdsp::game_data::BdspGameData;
pub use bdsp::pokemon::{BdspFactory, BdspPokemon};
pub use bdsp::save::SaveFile as BdspSaveFile;
pub use lumi::game_data::LumiGameData;
pub use lumi::pokemon::{LumiFactory, LumiPokemon};
pub use lumi::save::SaveFile as LumiSaveFile;
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
    /// A Pokémon from a BDSP save.
    Bdsp(BdspPokemon),
    /// A Pokémon from a Luminescent Platinum save.
    Lumi(LumiPokemon),
}

impl Pokemon for AnyPokemon {
    fn level(&self) -> u8 {
        match self {
            Self::Gen3(p) => p.level(),
            Self::Bdsp(p) => p.level(),
            Self::Lumi(p) => p.level(),
        }
    }
    fn set_level(&mut self, level: u8) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_level(level),
            Self::Bdsp(p) => p.set_level(level),
            Self::Lumi(p) => p.set_level(level),
        }
    }
    fn nature(&self) -> String {
        match self {
            Self::Gen3(p) => p.nature(),
            Self::Bdsp(p) => p.nature(),
            Self::Lumi(p) => p.nature(),
        }
    }
    fn set_nature(&mut self, nature: &str) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_nature(nature),
            Self::Bdsp(p) => p.set_nature(nature),
            Self::Lumi(p) => p.set_nature(nature),
        }
    }
    fn gender(&self) -> Gender {
        match self {
            Self::Gen3(p) => p.gender(),
            Self::Bdsp(p) => p.gender(),
            Self::Lumi(p) => p.gender(),
        }
    }
    fn set_gender(&mut self, gender: Gender) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_gender(gender),
            Self::Bdsp(p) => p.set_gender(gender),
            Self::Lumi(p) => p.set_gender(gender),
        }
    }
    fn species(&self) -> String {
        match self {
            Self::Gen3(p) => p.species(),
            Self::Bdsp(p) => p.species(),
            Self::Lumi(p) => p.species(),
        }
    }
    fn set_species(&mut self, species: &str) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_species(species),
            Self::Bdsp(p) => p.set_species(species),
            Self::Lumi(p) => p.set_species(species),
        }
    }
    fn ot_name(&self) -> String {
        match self {
            Self::Gen3(p) => p.ot_name(),
            Self::Bdsp(p) => p.ot_name(),
            Self::Lumi(p) => p.ot_name(),
        }
    }
    fn set_ot_name(&mut self, ot_name: &str) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_ot_name(ot_name),
            Self::Bdsp(p) => p.set_ot_name(ot_name),
            Self::Lumi(p) => p.set_ot_name(ot_name),
        }
    }
    fn ot_id(&self) -> TrainerID {
        match self {
            Self::Gen3(p) => p.ot_id(),
            Self::Bdsp(p) => p.ot_id(),
            Self::Lumi(p) => p.ot_id(),
        }
    }
    fn set_ot_id(&mut self, ot_id: &TrainerID) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_ot_id(ot_id),
            Self::Bdsp(p) => p.set_ot_id(ot_id),
            Self::Lumi(p) => p.set_ot_id(ot_id),
        }
    }
    fn nickname(&self) -> String {
        match self {
            Self::Gen3(p) => p.nickname(),
            Self::Bdsp(p) => p.nickname(),
            Self::Lumi(p) => p.nickname(),
        }
    }
    fn set_nickname(&mut self, nickname: &str) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_nickname(nickname),
            Self::Bdsp(p) => p.set_nickname(nickname),
            Self::Lumi(p) => p.set_nickname(nickname),
        }
    }
    fn held_item(&self) -> Option<String> {
        match self {
            Self::Gen3(p) => p.held_item(),
            Self::Bdsp(p) => p.held_item(),
            Self::Lumi(p) => p.held_item(),
        }
    }
    fn set_held_item(&mut self, item: &str) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_held_item(item),
            Self::Bdsp(p) => p.set_held_item(item),
            Self::Lumi(p) => p.set_held_item(item),
        }
    }
    fn moves(&self) -> Vec<Move> {
        match self {
            Self::Gen3(p) => p.moves(),
            Self::Bdsp(p) => p.moves(),
            Self::Lumi(p) => p.moves(),
        }
    }
    fn set_move(&mut self, slot: usize, attack: &str) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_move(slot, attack),
            Self::Bdsp(p) => p.set_move(slot, attack),
            Self::Lumi(p) => p.set_move(slot, attack),
        }
    }
    fn ability(&self) -> String {
        match self {
            Self::Gen3(p) => p.ability(),
            Self::Bdsp(p) => p.ability(),
            Self::Lumi(p) => p.ability(),
        }
    }
    fn set_ability(&mut self, ability: &str) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_ability(ability),
            Self::Bdsp(p) => p.set_ability(ability),
            Self::Lumi(p) => p.set_ability(ability),
        }
    }
    fn friendship(&self) -> u8 {
        match self {
            Self::Gen3(p) => p.friendship(),
            Self::Bdsp(p) => p.friendship(),
            Self::Lumi(p) => p.friendship(),
        }
    }
    fn set_friendship(&mut self, value: u8) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_friendship(value),
            Self::Bdsp(p) => p.set_friendship(value),
            Self::Lumi(p) => p.set_friendship(value),
        }
    }
    fn pokerus_status(&self) -> Pokerus {
        match self {
            Self::Gen3(p) => p.pokerus_status(),
            Self::Bdsp(p) => p.pokerus_status(),
            Self::Lumi(p) => p.pokerus_status(),
        }
    }
    fn set_pokerus_status(&mut self, status: Pokerus) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_pokerus_status(status),
            Self::Bdsp(p) => p.set_pokerus_status(status),
            Self::Lumi(p) => p.set_pokerus_status(status),
        }
    }
    fn ivs(&self) -> StatBlock {
        match self {
            Self::Gen3(p) => p.ivs(),
            Self::Bdsp(p) => p.ivs(),
            Self::Lumi(p) => p.ivs(),
        }
    }
    fn set_ivs(&mut self, ivs: StatBlock) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_ivs(ivs),
            Self::Bdsp(p) => p.set_ivs(ivs),
            Self::Lumi(p) => p.set_ivs(ivs),
        }
    }
    fn evs(&self) -> StatBlock {
        match self {
            Self::Gen3(p) => p.evs(),
            Self::Bdsp(p) => p.evs(),
            Self::Lumi(p) => p.evs(),
        }
    }
    fn set_evs(&mut self, evs: StatBlock) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_evs(evs),
            Self::Bdsp(p) => p.set_evs(evs),
            Self::Lumi(p) => p.set_evs(evs),
        }
    }
    fn exp(&self) -> u32 {
        match self {
            Self::Gen3(p) => p.exp(),
            Self::Bdsp(p) => p.exp(),
            Self::Lumi(p) => p.exp(),
        }
    }
    fn is_egg(&self) -> bool {
        match self {
            Self::Gen3(p) => p.is_egg(),
            Self::Bdsp(p) => p.is_egg(),
            Self::Lumi(p) => p.is_egg(),
        }
    }
    fn is_shiny(&self) -> bool {
        match self {
            Self::Gen3(p) => p.is_shiny(),
            Self::Bdsp(p) => p.is_shiny(),
            Self::Lumi(p) => p.is_shiny(),
        }
    }
    fn is_bad_egg(&self) -> bool {
        match self {
            Self::Gen3(p) => p.is_bad_egg(),
            Self::Bdsp(p) => p.is_bad_egg(),
            Self::Lumi(p) => p.is_bad_egg(),
        }
    }
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Gen3(p) => p.to_bytes(),
            Self::Bdsp(p) => p.to_bytes(),
            Self::Lumi(p) => p.to_bytes(),
        }
    }
    fn is_empty(&self) -> bool {
        match self {
            Self::Gen3(p) => p.is_empty(),
            Self::Bdsp(p) => p.is_empty(),
            Self::Lumi(p) => p.is_empty(),
        }
    }
    fn nat_dex_number(&self) -> u16 {
        match self {
            Self::Gen3(p) => p.nat_dex_number(),
            Self::Bdsp(p) => p.nat_dex_number(),
            Self::Lumi(p) => p.nat_dex_number(),
        }
    }
    fn form(&self) -> u8 {
        match self {
            Self::Gen3(p) => p.form(),
            Self::Bdsp(p) => p.form(),
            Self::Lumi(p) => p.form(),
        }
    }
    fn personality_value(&self) -> u32 {
        match self {
            Self::Gen3(p) => p.personality_value(),
            Self::Bdsp(p) => p.personality_value(),
            Self::Lumi(p) => p.personality_value(),
        }
    }
    fn language(&self) -> String {
        match self {
            Self::Gen3(p) => p.language(),
            Self::Bdsp(p) => p.language(),
            Self::Lumi(p) => p.language(),
        }
    }
    fn typing(&self) -> Option<(String, Option<String>)> {
        match self {
            Self::Gen3(p) => p.typing(),
            Self::Bdsp(p) => p.typing(),
            Self::Lumi(p) => p.typing(),
        }
    }
    fn pokeball_caught(&self) -> usize {
        match self {
            Self::Gen3(p) => p.pokeball_caught(),
            Self::Bdsp(p) => p.pokeball_caught(),
            Self::Lumi(p) => p.pokeball_caught(),
        }
    }
    fn set_pokeball_caught(&mut self, ball_id: u8) -> Result<(), PokemonError> {
        match self {
            Self::Gen3(p) => p.set_pokeball_caught(ball_id),
            Self::Bdsp(p) => p.set_pokeball_caught(ball_id),
            Self::Lumi(p) => p.set_pokeball_caught(ball_id),
        }
    }
    fn infect_pokerus(&mut self) {
        match self {
            Self::Gen3(p) => p.infect_pokerus(),
            Self::Bdsp(p) => p.infect_pokerus(),
            Self::Lumi(p) => p.infect_pokerus(),
        }
    }
    fn cure_pokerus(&mut self) {
        match self {
            Self::Gen3(p) => p.cure_pokerus(),
            Self::Bdsp(p) => p.cure_pokerus(),
            Self::Lumi(p) => p.cure_pokerus(),
        }
    }
    fn remove_pokerus(&mut self) {
        match self {
            Self::Gen3(p) => p.remove_pokerus(),
            Self::Bdsp(p) => p.remove_pokerus(),
            Self::Lumi(p) => p.remove_pokerus(),
        }
    }
    fn computed_stats(&self) -> ComputedStats {
        match self {
            Self::Gen3(p) => p.computed_stats(),
            Self::Bdsp(p) => p.computed_stats(),
            Self::Lumi(p) => p.computed_stats(),
        }
    }
    fn update_iv(&mut self, stat: &str, value: u16) {
        match self {
            Self::Gen3(p) => p.update_iv(stat, value),
            Self::Bdsp(p) => p.update_iv(stat, value),
            Self::Lumi(p) => p.update_iv(stat, value),
        }
    }
    fn update_ev(&mut self, stat: &str, value: u16) {
        match self {
            Self::Gen3(p) => p.update_ev(stat, value),
            Self::Bdsp(p) => p.update_ev(stat, value),
            Self::Lumi(p) => p.update_ev(stat, value),
        }
    }
    fn update_checksum(&mut self) {
        match self {
            Self::Gen3(p) => p.update_checksum(),
            Self::Bdsp(p) => p.update_checksum(),
            Self::Lumi(p) => p.update_checksum(),
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
    /// Game data for a BDSP save.
    Bdsp(BdspGameData),
    /// Game data for a Luminescent Platinum save.
    Lumi(LumiGameData),
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
            Self::Bdsp(g) => Ok(g.held_items()?),
            Self::Lumi(g) => Ok(g.held_items()?),
        }
    }
    fn nat_dex_num(&self, species: &str) -> Self::Result<u16> {
        match self {
            Self::Gen3(g) => Ok(g.nat_dex_num(species)?),
            Self::Bdsp(g) => Ok(g.nat_dex_num(species)?),
            Self::Lumi(g) => Ok(g.nat_dex_num(species)?),
        }
    }
    fn growth_rate(&self, dex_num: u16) -> Self::Result<String> {
        match self {
            Self::Gen3(g) => Ok(g.growth_rate(dex_num)?),
            Self::Bdsp(g) => Ok(g.growth_rate(dex_num)?),
            Self::Lumi(g) => Ok(g.growth_rate(dex_num)?),
        }
    }
    fn pk_species(&self, dex_num: u16) -> Self::Result<String> {
        match self {
            Self::Gen3(g) => Ok(g.pk_species(dex_num)?),
            Self::Bdsp(g) => Ok(g.pk_species(dex_num)?),
            Self::Lumi(g) => Ok(g.pk_species(dex_num)?),
        }
    }
    fn move_data(&self, id: usize) -> Self::Result<(String, String, u8)> {
        match self {
            Self::Gen3(g) => Ok(g.move_data(id)?),
            Self::Bdsp(g) => Ok(g.move_data(id)?),
            Self::Lumi(g) => Ok(g.move_data(id)?),
        }
    }
    fn typing(&self, dex_num: u16) -> Self::Result<(String, Option<String>)> {
        match self {
            Self::Gen3(g) => Ok(g.typing(dex_num)?),
            Self::Bdsp(g) => Ok(g.typing(dex_num)?),
            Self::Lumi(g) => Ok(g.typing(dex_num)?),
        }
    }
    fn typing_form(&self, dex_num: u16, form: u8) -> Self::Result<(String, Option<String>)> {
        match self {
            Self::Gen3(g) => Ok(g.typing_form(dex_num, form)?),
            Self::Bdsp(g) => Ok(g.typing_form(dex_num, form)?),
            Self::Lumi(g) => Ok(g.typing_form(dex_num, form)?),
        }
    }
    fn gender_ratio(&self, dex_num: u16) -> Self::Result<u8> {
        match self {
            Self::Gen3(g) => Ok(g.gender_ratio(dex_num)?),
            Self::Bdsp(g) => Ok(g.gender_ratio(dex_num)?),
            Self::Lumi(g) => Ok(g.gender_ratio(dex_num)?),
        }
    }
    fn species(&self) -> Self::Result<Vec<String>> {
        match self {
            Self::Gen3(g) => Ok(g.species()?),
            Self::Bdsp(g) => Ok(g.species()?),
            Self::Lumi(g) => Ok(g.species()?),
        }
    }
    fn moves(&self) -> Self::Result<Vec<String>> {
        match self {
            Self::Gen3(g) => Ok(g.moves()?),
            Self::Bdsp(g) => Ok(g.moves()?),
            Self::Lumi(g) => Ok(g.moves()?),
        }
    }
    fn base_stats(&self, dex_num: &u16) -> Self::Result<(u16, u16, u16, u16, u16, u16)> {
        match self {
            Self::Gen3(g) => Ok(g.base_stats(dex_num)?),
            Self::Bdsp(g) => Ok(g.base_stats(dex_num)?),
            Self::Lumi(g) => Ok(g.base_stats(dex_num)?),
        }
    }
    fn base_stats_form(
        &self,
        dex_num: u16,
        form: u8,
    ) -> Self::Result<(u16, u16, u16, u16, u16, u16)> {
        match self {
            Self::Gen3(g) => Ok(g.base_stats_form(dex_num, form)?),
            Self::Bdsp(g) => Ok(g.base_stats_form(dex_num, form)?),
            Self::Lumi(g) => Ok(g.base_stats_form(dex_num, form)?),
        }
    }
    fn evolution(&self, dex_num: &u16) -> Self::Result<Evolution> {
        match self {
            Self::Gen3(g) => Ok(g.evolution(dex_num)?),
            Self::Bdsp(g) => Ok(g.evolution(dex_num)?),
            Self::Lumi(g) => Ok(g.evolution(dex_num)?),
        }
    }
    fn abilities(&self, dex_num: u16) -> Self::Result<Abilities> {
        match self {
            Self::Gen3(g) => Ok(g.abilities(dex_num)?),
            Self::Bdsp(g) => Ok(g.abilities(dex_num)?),
            Self::Lumi(g) => Ok(g.abilities(dex_num)?),
        }
    }
    fn abilities_form(&self, dex_num: u16, form: u8) -> Self::Result<Abilities> {
        match self {
            Self::Gen3(g) => Ok(g.abilities_form(dex_num, form)?),
            Self::Bdsp(g) => Ok(g.abilities_form(dex_num, form)?),
            Self::Lumi(g) => Ok(g.abilities_form(dex_num, form)?),
        }
    }
    fn ability_ids(&self, dex_num: u16) -> Self::Result<(u16, Option<u16>, Option<u16>)> {
        match self {
            Self::Gen3(g) => Ok(g.ability_ids(dex_num)?),
            Self::Bdsp(g) => Ok(g.ability_ids(dex_num)?),
            Self::Lumi(g) => Ok(g.ability_ids(dex_num)?),
        }
    }
    fn ability_ids_form(
        &self,
        dex_num: u16,
        form: u8,
    ) -> Self::Result<(u16, Option<u16>, Option<u16>)> {
        match self {
            Self::Gen3(g) => Ok(g.ability_ids_form(dex_num, form)?),
            Self::Bdsp(g) => Ok(g.ability_ids_form(dex_num, form)?),
            Self::Lumi(g) => Ok(g.ability_ids_form(dex_num, form)?),
        }
    }
    fn items_in_pocket(&self, pocket: Pocket) -> Self::Result<Vec<String>> {
        match self {
            Self::Gen3(g) => Ok(g.items_in_pocket(pocket)?),
            Self::Bdsp(g) => Ok(g.items_in_pocket(pocket)?),
            Self::Lumi(g) => Ok(g.items_in_pocket(pocket)?),
        }
    }
    fn item_flavor_text(&self, name: &str) -> String {
        match self {
            Self::Gen3(g) => g.item_flavor_text(name),
            Self::Bdsp(g) => g.item_flavor_text(name),
            Self::Lumi(g) => g.item_flavor_text(name),
        }
    }
    fn balls_id(&self) -> Self::Result<Vec<u16>> {
        match self {
            Self::Gen3(g) => Ok(g.balls_id()?),
            Self::Bdsp(g) => Ok(g.balls_id()?),
            Self::Lumi(g) => Ok(g.balls_id()?),
        }
    }
    fn item_id_by_name(&self, name: &str) -> Self::Result<usize> {
        match self {
            Self::Gen3(g) => Ok(g.item_id_by_name(name)?),
            Self::Bdsp(g) => Ok(g.item_id_by_name(name)?),
            Self::Lumi(g) => Ok(g.item_id_by_name(name)?),
        }
    }
    fn item_sprite_id(&self, name: &str) -> Self::Result<usize> {
        match self {
            Self::Gen3(g) => Ok(g.item_sprite_id(name)?),
            Self::Bdsp(g) => Ok(g.item_sprite_id(name)?),
            Self::Lumi(g) => Ok(g.item_sprite_id(name)?),
        }
    }
    fn balls_sprite_ids(&self) -> Self::Result<Vec<u16>> {
        match self {
            Self::Gen3(g) => Ok(g.balls_sprite_ids()?),
            Self::Bdsp(g) => Ok(g.balls_sprite_ids()?),
            Self::Lumi(g) => Ok(g.balls_sprite_ids()?),
        }
    }
    fn lowest_level(&self, dex_num: u16) -> u8 {
        match self {
            Self::Gen3(g) => g.lowest_level(dex_num),
            Self::Bdsp(g) => g.lowest_level(dex_num),
            Self::Lumi(g) => g.lowest_level(dex_num),
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
    /// Factory for BDSP Pokémon.
    Bdsp(BdspFactory),
    /// Factory for Luminescent Platinum Pokémon.
    Lumi(LumiFactory),
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
        pokemon: &AnyPokemon,
        species: &str,
        ot_name: &str,
        ot_id: TrainerID,
    ) -> Result<AnyPokemon, error::PokemonError> {
        match self {
            Self::Gen3(f) => match pokemon {
                AnyPokemon::Gen3(p) => f
                    .gen_pokemon_from_species(p, species, ot_name, ot_id)
                    .map(AnyPokemon::Gen3),
                AnyPokemon::Bdsp(_) | AnyPokemon::Lumi(_) => {
                    Err(error::PokemonError::UnknownSpecies(species.to_string()))
                }
            },
            Self::Bdsp(f) => match pokemon {
                AnyPokemon::Bdsp(p) => f
                    .gen_pokemon_from_species(p, species, ot_name, ot_id)
                    .map(AnyPokemon::Bdsp),
                AnyPokemon::Gen3(_) | AnyPokemon::Lumi(_) => {
                    Err(error::PokemonError::UnknownSpecies(species.to_string()))
                }
            },
            Self::Lumi(f) => match pokemon {
                AnyPokemon::Lumi(p) => f
                    .gen_pokemon_from_species(p, species, ot_name, ot_id)
                    .map(AnyPokemon::Lumi),
                AnyPokemon::Gen3(_) | AnyPokemon::Bdsp(_) => {
                    Err(error::PokemonError::UnknownSpecies(species.to_string()))
                }
            },
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
    /// A Generation III save (`Ruby`, `Sapphire`, `Emerald`, `FireRed`, `LeafGreen`).
    Gen3(gen3::save::SaveFile),
    /// A BDSP save (`Brilliant Diamond`, `Shining Pearl`).
    Bdsp(bdsp::save::SaveFile),
    /// A Luminescent Platinum save.
    Lumi(lumi::save::SaveFile),
}

/// Converts a badge *count* into a bitmask with the lowest `count` bits set.
///
/// Used for formats where the per-badge storage is not yet decoded, so the
/// count still renders as the expected number of earned badges.
fn count_to_flags(count: u8) -> u8 {
    match count.min(8) {
        8 => u8::MAX,
        n => (1u8 << n).wrapping_sub(1),
    }
}

impl OpenSave {
    /// Returns the [`AnyGameData`] dispatcher that matches this save's generation.
    pub fn game_data(&self) -> AnyGameData {
        match self {
            Self::Gen3(_) => AnyGameData::Gen3(Gen3GameData),
            Self::Bdsp(_) => AnyGameData::Bdsp(BdspGameData),
            Self::Lumi(_) => AnyGameData::Lumi(LumiGameData),
        }
    }

    /// Returns the [`AnyFactory`] for creating new Pokémon in this save's generation.
    pub fn pokemon_factory(&self) -> AnyFactory {
        match self {
            Self::Gen3(_) => AnyFactory::Gen3(Gen3Factory),
            Self::Bdsp(_) => AnyFactory::Bdsp(BdspFactory),
            Self::Lumi(_) => AnyFactory::Lumi(LumiFactory),
        }
    }

    /// Returns the party Pokémon as [`AnyPokemon`] values.
    ///
    /// # Errors
    /// Returns an error if the party section cannot be parsed.
    pub fn party(&self) -> Result<Vec<AnyPokemon>, Box<dyn std::error::Error>> {
        match self {
            Self::Gen3(s) => Ok(s.get_party()?.into_iter().map(AnyPokemon::Gen3).collect()),
            Self::Bdsp(s) => Ok(s.party()?.into_iter().map(AnyPokemon::Bdsp).collect()),
            Self::Lumi(s) => Ok(s.party()?.into_iter().map(AnyPokemon::Lumi).collect()),
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
            Self::Bdsp(s) => Ok(s.pc_box(number)?.into_iter().map(AnyPokemon::Bdsp).collect()),
            Self::Lumi(s) => Ok(s.pc_box(number)?.into_iter().map(AnyPokemon::Lumi).collect()),
        }
    }

    /// Returns `true` if the save file buffer is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Gen3(s) => s.is_empty(),
            Self::Bdsp(s) => s.is_empty(),
            Self::Lumi(s) => s.is_empty(),
        }
    }

    /// Returns `true` if the PC buffer contains no Pokémon.
    pub fn is_pc_empty(&self) -> bool {
        match self {
            Self::Gen3(s) => s.is_pc_empty(),
            Self::Bdsp(s) => s.is_pc_empty(),
            Self::Lumi(s) => s.is_pc_empty(),
        }
    }

    /// Returns the number of gym badges earned (0–8), or `None` for unsupported formats.
    pub fn badge_count(&self) -> Option<u8> {
        match self {
            Self::Gen3(s) => s.badges().ok().map(|b| b.count()),
            Self::Bdsp(s) => Some(s.badge_count()),
            Self::Lumi(s) => Some(s.badge_count()),
        }
    }

    /// Returns the earned badges as a bitmask (bit `N` = badge `N + 1`), or
    /// `None` for unsupported formats.
    ///
    /// For BDSP/Lumi the underlying store is not yet decoded per-badge, so the
    /// count is mapped onto the lowest `count` bits to keep the displayed total
    /// correct.
    pub fn badge_flags(&self) -> Option<u8> {
        match self {
            Self::Gen3(s) => s.badges().ok().map(|b| b.flags()),
            Self::Bdsp(s) => Some(count_to_flags(s.badge_count())),
            Self::Lumi(s) => Some(count_to_flags(s.badge_count())),
        }
    }

    /// Returns `(caught, seen)` Pokédex counts, or `None` for unsupported formats.
    pub fn pokedex_counts(&self) -> Option<(u16, u16)> {
        match self {
            Self::Gen3(s) => s.pokedex().ok(),
            Self::Bdsp(s) => Some(s.pokedex_counts()),
            Self::Lumi(s) => Some(s.pokedex_counts()),
        }
    }

    /// Returns a copy of the raw save file bytes.
    pub fn raw_data(&self) -> Vec<u8> {
        match self {
            Self::Gen3(s) => s.raw_data(),
            Self::Bdsp(s) => s.raw_data(),
            Self::Lumi(s) => s.raw_data(),
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
        let common_storage = match storage {
            gen3::save::storage::StorageType::PC => common::types::StorageType::PC,
            gen3::save::storage::StorageType::Party => common::types::StorageType::Party,
            gen3::save::storage::StorageType::None => common::types::StorageType::None,
        };
        match (self, pokemon) {
            (Self::Gen3(s), AnyPokemon::Gen3(p)) => Ok(s.save_pokemon(storage, p)?),
            (Self::Bdsp(s), AnyPokemon::Bdsp(p)) => Ok(s.save_pokemon(common_storage, p)?),
            (Self::Lumi(s), AnyPokemon::Lumi(p)) => Ok(s.save_pokemon(common_storage, p)?),
            _ => Err("Pokémon generation does not match save file".into()),
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
        let common_from = match from_storage {
            gen3::save::storage::StorageType::PC => common::types::StorageType::PC,
            gen3::save::storage::StorageType::Party => common::types::StorageType::Party,
            gen3::save::storage::StorageType::None => common::types::StorageType::None,
        };
        let common_to = match to_storage {
            gen3::save::storage::StorageType::PC => common::types::StorageType::PC,
            gen3::save::storage::StorageType::Party => common::types::StorageType::Party,
            gen3::save::storage::StorageType::None => common::types::StorageType::None,
        };
        match (self, from, to) {
            (Self::Gen3(s), AnyPokemon::Gen3(f), AnyPokemon::Gen3(t)) => {
                Ok(s.swap_pokemon(f, from_storage, t, to_storage)?)
            }
            (Self::Bdsp(s), AnyPokemon::Bdsp(f), AnyPokemon::Bdsp(t)) => {
                Ok(s.swap_pokemon(f, common_from, t, common_to)?)
            }
            (Self::Lumi(s), AnyPokemon::Lumi(f), AnyPokemon::Lumi(t)) => {
                Ok(s.swap_pokemon(f, common_from, t, common_to)?)
            }
            _ => Err("Pokémon generation does not match save file".into()),
        }
    }

    /// Returns the full [`Trainer`] metadata struct.
    ///
    /// # Errors
    /// Returns an error if the trainer section cannot be read.
    pub fn trainer(&self) -> Result<Trainer, error::SaveDataError> {
        match self {
            Self::Gen3(s) => s.trainer(),
            Self::Bdsp(s) => s.trainer().map(Into::into),
            Self::Lumi(s) => s.trainer().map(Into::into),
        }
    }

    /// Saves updated [`Trainer`] data back into the save file.
    ///
    /// # Errors
    /// Returns an error if writing fails or a required section is missing.
    pub fn save_trainer(&mut self, trainer: &Trainer) -> Result<(), error::SaveDataError> {
        let common = common::types::Trainer::from(trainer.clone());
        match self {
            Self::Gen3(s) => s.save_trainer(trainer),
            Self::Bdsp(s) => s.save_trainer(&common),
            Self::Lumi(s) => s.save_trainer(&common),
        }
    }

    /// Returns the trainer's name decoded from the save file.
    ///
    /// Returns an empty string if the trainer section cannot be read.
    pub fn trainer_name(&self) -> String {
        match self {
            Self::Gen3(s) => s.trainer().map(|t| t.name).unwrap_or_default(),
            Self::Bdsp(s) => s.trainer().map(|t| t.name).unwrap_or_default(),
            Self::Lumi(s) => s.trainer().map(|t| t.name).unwrap_or_default(),
        }
    }

    /// Returns the trainer's ID from the save file.
    ///
    /// Returns a default [`TrainerID`] if the trainer section cannot be read.
    pub fn trainer_id(&self) -> TrainerID {
        match self {
            Self::Gen3(s) => s.trainer().map(|t| t.id).unwrap_or_default(),
            Self::Bdsp(s) => s.trainer().map(|t| t.id).unwrap_or_default(),
            Self::Lumi(s) => s.trainer().map(|t| t.id).unwrap_or_default(),
        }
    }

    /// Returns the contents of the requested bag pocket.
    ///
    /// Each entry is `(item_name, quantity)`.
    ///
    /// # Errors
    /// Returns an error if the pocket data cannot be decoded.
    pub fn pocket(&self, pocket: Pocket) -> Result<Vec<BagItem>, error::SaveDataError> {
        match self {
            Self::Gen3(s) => s.pocket(pocket.try_into()?),
            Self::Bdsp(s) => s.pocket(pocket),
            Self::Lumi(s) => s.pocket(pocket),
        }
    }

    /// Writes updated bag pocket data back into the save file.
    ///
    /// # Errors
    /// Returns an error if the pocket data cannot be encoded or the section is missing.
    pub fn save_pocket(
        &mut self,
        pocket_type: Pocket,
        pocket_list: Vec<BagItem>,
    ) -> Result<(), error::SaveDataError> {
        match self {
            Self::Gen3(s) => s.save_pocket(pocket_type.try_into()?, pocket_list),
            Self::Bdsp(s) => s.save_pocket(pocket_type, pocket_list),
            Self::Lumi(s) => s.save_pocket(pocket_type, pocket_list),
        }
    }

    /// Returns the bag pockets this save's generation presents, in tab order.
    pub fn pockets(&self) -> &'static [Pocket] {
        match self {
            Self::Gen3(_) => Pocket::GEN3,
            Self::Bdsp(_) | Self::Lumi(_) => Pocket::BDSP,
        }
    }

    /// Returns the maximum stack size for a single item slot in `pocket`.
    pub fn max_quantity(&self, pocket: Pocket) -> u16 {
        if !pocket.has_quantity() {
            return 1;
        }
        match self {
            Self::Gen3(_) => 99,
            Self::Bdsp(_) | Self::Lumi(_) => common::inventory8b::MAX_QUANTITY,
        }
    }

    /// Returns `true` if pockets hold a fixed number of slots.
    ///
    /// Gen III reserves a fixed slot list per pocket, so empty slots are part of
    /// the data and an item is "removed" by blanking its slot in place. BDSP and
    /// Lumi instead store only the items the player owns, so a pocket grows and
    /// shrinks as items are added and removed.
    pub fn has_fixed_pocket_slots(&self) -> bool {
        matches!(self, Self::Gen3(_))
    }
}

/// Opens a raw save file buffer, detects the generation, and returns an [`OpenSave`].
///
/// Detection order:
/// 1. **Lumi** — file size matches a Lumi size (`0xEDC20` or `0xEF0A4`) and the
///    `u32` at offset 0 has the `0xFFFF0000` prefix.
/// 2. **BDSP** — file size matches a BDSP size and the `u32` at offset 0 is a
///    valid game version (`0x25`–`0x34`).
/// 3. **Gen3** — 128 KB save.
///
/// # Errors
/// Returns [`DetectError::TooSmall`] if the buffer is under 128 bytes.
/// Returns [`DetectError::UnknownFormat`] if no supported format matches.
/// Returns [`DetectError::ChecksumFailed`] if any section checksum is invalid.
pub fn open(data: &[u8]) -> Result<OpenSave, DetectError> {
    const GEN3_SIZE: usize = 131_072; // 128 KB

    if data.len() < 128 {
        return Err(DetectError::TooSmall(data.len()));
    }

    let version_marker = data.get(0..4).map(LittleEndian::read_u32).unwrap_or(0);

    // Lumi first — check 0xFFFF prefix before BDSP (order matters)
    if matches!(data.len(), 0xEDC20 | 0xEF0A4) && (version_marker & 0xFFFF_0000) == 0xFFFF_0000 {
        let save = lumi::save::SaveFile::new(data).map_err(DetectError::ChecksumFailed)?;
        return Ok(OpenSave::Lumi(save));
    }

    // BDSP — version must be a valid Gem8Version
    if matches!(
        version_marker,
        0x25 | 0x2C | 0x32 | 0x34
    ) && matches!(data.len(), 0xE9828 | 0xEDC20 | 0xEED8C | 0xEF0A4)
    {
        let save = bdsp::save::SaveFile::new(data).map_err(DetectError::ChecksumFailed)?;
        return Ok(OpenSave::Bdsp(save));
    }

    if data.len() >= GEN3_SIZE {
        let save = gen3::save::SaveFile::new(data).map_err(DetectError::ChecksumFailed)?;
        return Ok(OpenSave::Gen3(save));
    }

    Err(DetectError::UnknownFormat(data.len()))
}

#[cfg(test)]
mod badge_flag_tests {
    use super::count_to_flags;

    #[test]
    fn count_maps_to_lowest_bits() {
        assert_eq!(count_to_flags(0), 0b0000_0000);
        assert_eq!(count_to_flags(1), 0b0000_0001);
        assert_eq!(count_to_flags(3), 0b0000_0111);
        assert_eq!(count_to_flags(5), 0b0001_1111);
        assert_eq!(count_to_flags(8), 0b1111_1111);
    }

    #[test]
    fn count_saturates_past_eight() {
        assert_eq!(count_to_flags(9), 0b1111_1111);
        assert_eq!(count_to_flags(u8::MAX), 0b1111_1111);
    }
}
