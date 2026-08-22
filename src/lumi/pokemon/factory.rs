use super::pokemon::LumiPokemon;
use crate::lumi::game_data::LumiGameData;
use crate::common::types::TrainerID;
use crate::error::PokemonError;
use crate::traits::game_data::GameData;
use crate::traits::pokemon::Pokemon;
use crate::traits::pokemon_factory::PokemonFactory;
use byteorder::{ByteOrder, LittleEndian};
use rand::Rng;

const VERSION_BRILLIANT_DIAMOND: u8 = 48;
const LOCATION_NONE: u16 = 0xFFFF;

#[derive(Debug, Clone, Copy, Default)]
pub struct LumiFactory;

impl PokemonFactory for LumiFactory {
    type Output = LumiPokemon;

    fn gen_pokemon_from_species(
        &self,
        pokemon: &LumiPokemon,
        species: &str,
        ot_name: &str,
        ot_id: TrainerID,
    ) -> Result<LumiPokemon, PokemonError> {
        let dex_num = LumiGameData
            .nat_dex_num(species)
            .map_err(|_| PokemonError::UnknownSpecies(species.to_string()))?;

        let mut new_pokemon = LumiPokemon::new();
        new_pokemon.offset = pokemon.offset;
        new_pokemon.set_species(species)?;
        new_pokemon.set_ot_name(ot_name)?;
        new_pokemon.set_ot_id(&ot_id)?;
        new_pokemon.set_nickname(species)?;

        let mut rng = rand::thread_rng();
        let ec: u32 = rng.gen();
        let pid: u32 = rng.gen();
        if let Some(dest) = new_pokemon.data.get_mut(0x00..0x04) {
            LittleEndian::write_u32(dest, ec);
        }
        if let Some(dest) = new_pokemon.data.get_mut(0x1C..0x20) {
            LittleEndian::write_u32(dest, pid);
        }

        let level = LumiGameData.lowest_level(dex_num).max(1);

        let ivs = crate::common::types::StatBlock {
            hp: rng.gen_range(0..32),
            attack: rng.gen_range(0..32),
            defense: rng.gen_range(0..32),
            speed: rng.gen_range(0..32),
            special_attack: rng.gen_range(0..32),
            special_defense: rng.gen_range(0..32),
        };
        new_pokemon.set_ivs(ivs)?;

        if let Some(dest) = new_pokemon.data.get_mut(0x124) {
            *dest = 4; // Poké Ball
        }
        // Met level occupies the low 7 bits of 0x125; bit 7 is OT gender.
        if let Some(dest) = new_pokemon.data.get_mut(0x125) {
            *dest = level & 0x7F; // Met level, OT gender = male (bit 7 = 0)
        }
        // Version (game of origin) lives at 0x0DE, NOT 0x126 (which is
        // HyperTrainFlags). Writing the wrong offset made the game misread the
        // origin (e.g. showing "Arceus").
        if let Some(dest) = new_pokemon.data.get_mut(0xDE) {
            *dest = VERSION_BRILLIANT_DIAMOND;
        }
        if let Some(dest) = new_pokemon.data.get_mut(0xE2) {
            *dest = 2; // Language: English
        }
        // Affixed ribbon (-1 = none) is a signed byte at 0xE8. The old code
        // wrote 0xFF to 0x40, which is the ribbon flag block, granting bogus
        // ribbons instead.
        if let Some(dest) = new_pokemon.data.get_mut(0xE8) {
            *dest = 0xFF; // -1: no affixed ribbon
        }
        // Egg/Met locations default to the "none" sentinel (0xFFFF).
        if let Some(dest) = new_pokemon.data.get_mut(0x120..0x122) {
            LittleEndian::write_u16(dest, LOCATION_NONE);
        }
        if let Some(dest) = new_pokemon.data.get_mut(0x122..0x124) {
            LittleEndian::write_u16(dest, LOCATION_NONE);
        }
        // Ability: PB8 stores the ability ID (u16 @ 0x14) and the slot number
        // (0x16 & 7) separately. Default to slot 1 and its matching ID.
        let (ability1_id, _, _) = LumiGameData.ability_ids(dex_num).unwrap_or((0, None, None));
        if let Some(dest) = new_pokemon.data.get_mut(0x14..0x16) {
            LittleEndian::write_u16(dest, ability1_id);
        }
        if let Some(dest) = new_pokemon.data.get_mut(0x16) {
            *dest &= 0xF8; // Ability slot 0 (first ability)
        }
        if let Some(dest) = new_pokemon.data.get_mut(0x22) {
            *dest = 0; // Male by default
        }

        // Set level last so the party stat block (and current HP) is recomputed
        // from the final species/IVs; otherwise the Pokémon reads as fainted.
        new_pokemon.set_level(level)?;
        new_pokemon.update_checksum();
        Ok(new_pokemon)
    }
}
