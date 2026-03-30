//! Pokémon generation helpers using the Method 1 RNG algorithm.
//!
//! Generation III uses a linear congruential generator (LCG) with multiplier
//! `0x41C64E6D` and increment `0x6073` to produce legal PIDs and IVs for
//! wild encounters ("Method 1"). [`generate_method_1`] replicates this sequence
//! and [`gen_pokemon_from_species`] uses it to create a fully initialised Pokémon.

use crate::error::PokemonError;
use crate::pokemon::data::IVsEggAbility;
use crate::pokemon::Pokemon;
use rand::Rng;

const MULTIPLIER: u32 = 0x41C64E6D;
const INCREMENT: u32 = 0x6073;

/// Advances the LCG state by one step and returns the upper 16 bits of the result.
pub fn rng(state: &mut u32) -> u32 {
    *state = state.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT);
    *state >> 16
}

/// Generates a legal PID and IVs using the "Method 1" wild-encounter RNG sequence.
///
/// Runs four LCG steps: PID-low → PID-high → IVs-1 (HP/Atk/Def) → IVs-2 (Spd/SpAtk/SpDef).
/// Pass `None` for `seed` to use a random starting value via `rand`.
///
/// Returns `(PID, IVsEggAbility)`.
pub fn generate_method_1(seed: Option<u32>) -> (u32, IVsEggAbility) {
    let mut state = seed.unwrap_or_else(|| rand::thread_rng().gen());

    // Method 1 Sequence:
    // 1. PID Low (16 bits)
    // 2. PID High (16 bits)
    // 3. IVs 1 (HP, Atk, Def)
    // 4. IVs 2 (Spd, SpAtk, SpDef)

    let pid_low = rng(&mut state);
    let pid_high = rng(&mut state);
    let pid = pid_low | (pid_high << 16);

    let ivs_1 = rng(&mut state);
    let ivs_2 = rng(&mut state);

    // Unpack IVs from the 16-bit chunks
    // IVs 1: [Def:5][Atk:5][HP:5][x:1]
    let hp = ivs_1 & 0x1F;
    let atk = (ivs_1 >> 5) & 0x1F;
    let def = (ivs_1 >> 10) & 0x1F;

    // IVs 2: [SpDef:5][SpAtk:5][Spd:5][x:1]
    let spd = ivs_2 & 0x1F;
    let spa = (ivs_2 >> 5) & 0x1F;
    let spdef = (ivs_2 >> 10) & 0x1F;

    let mut iv_data = IVsEggAbility::new();
    iv_data.set_hp_iv(hp as u8);
    iv_data.set_attack_iv(atk as u8);
    iv_data.set_defense_iv(def as u8);
    iv_data.set_speed_iv(spd as u8);
    iv_data.set_sp_attack_iv(spa as u8);
    iv_data.set_sp_defense_iv(spdef as u8);

    // Ability bit is usually 0 or 1 depending on the RNG,
    // but simplified here (logic varies by game/encounter).
    // A common approximation is using the bit from the PID or RNG state.
    iv_data.set_ability_bit((pid & 1) as u8);

    (pid, iv_data)
}

/// Creates a new [`Pokemon`] of the given species for the specified trainer.
///
/// Generates a Method 1 PID and IVs, sets the species (auto-updating the nickname and level),
/// assigns the trainer's OT name and ID, and recalculates the checksum.
///
/// # Errors
/// Returns [`PokemonError::UnknownSpecies`] if `species` is not in the Pokédex database.
pub fn gen_pokemon_from_species(
    mut new_pokemon: Pokemon,
    species: &str,
    ot_name: &[u8],
    ot_id: &[u8],
) -> Result<Pokemon, PokemonError> {
    // 1. Generate PID and IVs together
    let (pid, ivs) = generate_method_1(None);

    new_pokemon.personality_value = pid;
    new_pokemon.data.misc.iv_egg_ability = ivs;

    new_pokemon.set_species(species)?;
    new_pokemon.set_level(new_pokemon.lowest_level());
    new_pokemon.data.misc.origins_info.set_pokeball(4);
    new_pokemon.set_ot_id(ot_id);
    new_pokemon.set_ot_name(ot_name);
    new_pokemon.set_nickname(&species.to_uppercase());

    new_pokemon.update_checksum();

    Ok(new_pokemon)
}
