use crate::common::types::{ComputedStats, Gender, Move, Pokerus, StatBlock, TrainerID};
use crate::error::PokemonError;
use crate::bdsp::game_data::BdspGameData;
use crate::bdsp::pokemon::crypto;
use crate::bdsp::pokemon::data::Stats;
use crate::traits::game_data::GameData;
use crate::traits::pokemon::Pokemon;
use byteorder::{ByteOrder, LittleEndian};
use rusqlite::Connection;

const SIZE_PARTY: usize = crypto::SIZE_PARTY;

fn read_utf16le(data: &[u8], offset: usize, max_bytes: usize) -> String {
    let mut chars = Vec::new();
    let mut i = 0;
    while i + 1 < max_bytes {
        let Some(byte_pair) = data.get(offset + i..offset + i + 2) else {
            break;
        };
        let ch = LittleEndian::read_u16(byte_pair);
        if ch == 0 {
            break;
        }
        chars.push(ch);
        i += 2;
    }
    String::from_utf16_lossy(&chars)
}

fn write_utf16le(data: &mut [u8], offset: usize, text: &str, max_chars: usize) {
    for (i, ch) in text.chars().take(max_chars).enumerate() {
        let Some(dest) = data.get_mut(offset + i * 2..offset + i * 2 + 2) else {
            break;
        };
        LittleEndian::write_u16(dest, ch as u16);
    }
    let term_pos = offset + text.chars().count().min(max_chars) * 2;
    if let Some(dest) = data.get_mut(term_pos..term_pos + 2) {
        LittleEndian::write_u16(dest, 0);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BdspPokemon {
    pub offset: usize,
    pub data: [u8; SIZE_PARTY],
    pub(crate) stats: Stats,
}

impl BdspPokemon {
    pub fn new() -> Self {
        Self {
            offset: 0,
            data: [0u8; SIZE_PARTY],
            stats: Stats::default(),
        }
    }

    pub fn from_bytes(offset: usize, raw: &[u8]) -> Result<Self, PokemonError> {
        if raw.len() < SIZE_PARTY {
            return Err(PokemonError::InvalidDataLength(raw.len()));
        }

        let mut buf = raw[..SIZE_PARTY].to_vec();
        if crypto::is_encrypted(&buf) {
            buf = crypto::decrypt(&buf);
            if buf.len() < SIZE_PARTY {
                return Err(PokemonError::InvalidDataLength(buf.len()));
            }
        }

        let mut data = [0u8; SIZE_PARTY];
        data.copy_from_slice(&buf[..SIZE_PARTY]);

        let mut pokemon = BdspPokemon {
            offset,
            data,
            stats: Stats::default(),
        };
        pokemon.init_stats();
        Ok(pokemon)
    }

    pub fn to_bytes_encrypted(&self) -> Vec<u8> {
        let mut data = self.data;
        let chk = crypto::calculate_checksum(&data);
        if let Some(dest) = data.get_mut(0x06..0x08) {
            LittleEndian::write_u16(dest, chk);
        }
        crypto::encrypt(&data)
    }

    pub fn species_id(&self) -> u16 {
        LittleEndian::read_u16(self.data.get(0x08..0x0A).unwrap_or(&[0u8; 2]))
    }

    /// Sets the alternate-form index at `0x24`.
    pub fn set_form(&mut self, form: u8) {
        if let Some(dest) = self.data.get_mut(0x24..0x26) {
            LittleEndian::write_u16(dest, u16::from(form));
        }
        self.init_stats();
    }

    pub fn experience(&self) -> u32 {
        LittleEndian::read_u32(self.data.get(0x10..0x14).unwrap_or(&[0u8; 4]))
    }

    pub fn init_stats(&mut self) {
        let dex = self.nat_dex_number();
        let form = self.form();
        let (base_hp, base_atk, base_def, base_spa, base_spd, base_spe) = BdspGameData
            .base_stats_form(dex, form)
            .unwrap_or((0, 0, 0, 0, 0, 0));
        let ivs = self.ivs();
        let evs = self.evs();
        let nature = self.data.get(0x21).copied().unwrap_or(0) as usize;

        self.stats = Stats {
            base_hp: base_hp as u16,
            base_attack: base_atk as u16,
            base_defense: base_def as u16,
            base_sp_attack: base_spa as u16,
            base_sp_defense: base_spd as u16,
            base_speed: base_spe as u16,
            hp_iv: ivs.hp as u16,
            attack_iv: ivs.attack as u16,
            defense_iv: ivs.defense as u16,
            sp_attack_iv: ivs.special_attack as u16,
            sp_defense_iv: ivs.special_defense as u16,
            speed_iv: ivs.speed as u16,
            hp_ev: evs.hp as u16,
            attack_ev: evs.attack as u16,
            defense_ev: evs.defense as u16,
            sp_attack_ev: evs.special_attack as u16,
            sp_defense_ev: evs.special_defense as u16,
            speed_ev: evs.speed as u16,
            n_mod: crate::gen3::game_data::NATURE_MODIFIER[nature.min(24)],
        };

        self.recompute_party_stats();
    }

    /// Writes the party-only battle stat block (`0x148`–`0x156`) and the current
    /// HP (`0x8A`) from the computed stats.
    ///
    /// The game reads these cached values directly for out-of-battle display, so
    /// a freshly created Pokémon whose block is left zeroed reads back as fainted
    /// (current HP 0) with a level-0 stat line. Current HP is set to the full
    /// max so a new Pokémon starts healthy.
    pub fn recompute_party_stats(&mut self) {
        if self.is_empty() {
            return;
        }
        let level = self.level();
        let hp_max = self.stats.hp(level);
        let atk = self.stats.attack(level);
        let def = self.stats.defense(level);
        let spe = self.stats.speed(level);
        let spa = self.stats.sp_attack(level);
        let spd = self.stats.sp_defense(level);

        if let Some(dest) = self.data.get_mut(0x8A..0x8C) {
            LittleEndian::write_u16(dest, hp_max);
        }
        if let Some(dest) = self.data.get_mut(0x148) {
            *dest = level;
        }
        for (offset, value) in [
            (0x14A, hp_max),
            (0x14C, atk),
            (0x14E, def),
            (0x150, spe),
            (0x152, spa),
            (0x154, spd),
        ] {
            if let Some(dest) = self.data.get_mut(offset..offset + 2) {
                LittleEndian::write_u16(dest, value);
            }
        }
    }
}

impl Pokemon for BdspPokemon {
    fn level(&self) -> u8 {
        self.data.get(0x148).copied().unwrap_or(1).clamp(1, 100)
    }

    fn set_level(&mut self, level: u8) -> Result<(), PokemonError> {
        let level = level.clamp(1, 100);
        if let Some(dest) = self.data.get_mut(0x148) {
            *dest = level;
        }
        self.update_exp_for_level(level);
        self.init_stats();
        Ok(())
    }

    fn nature(&self) -> String {
        if self.is_empty() {
            return String::new();
        }
        let idx = self.data.get(0x20).copied().unwrap_or(0) as usize;
        crate::gen3::game_data::NATURE.get(idx).copied().unwrap_or("Hardy").to_string()
    }

    fn set_nature(&mut self, nature: &str) -> Result<(), PokemonError> {
        if let Some(idx) = crate::gen3::game_data::NATURE.iter().position(|&n| n == nature) {
            if let Some(dest) = self.data.get_mut(0x20) {
                *dest = idx as u8;
            }
            if let Some(dest) = self.data.get_mut(0x21) {
                *dest = idx as u8;
            }
            self.init_stats();
        }
        Ok(())
    }

    fn gender(&self) -> Gender {
        let flags = self.data.get(0x22).copied().unwrap_or(0);
        match (flags >> 2) & 0x03 {
            0 => Gender::M,
            1 => Gender::F,
            2 => Gender::None,
            _ => Gender::M,
        }
    }

    fn set_gender(&mut self, gender: Gender) -> Result<(), PokemonError> {
        let val = match gender {
            Gender::M => 0u8,
            Gender::F => 1,
            Gender::None => 2,
        };
        if let Some(dest) = self.data.get_mut(0x22) {
            *dest = (*dest & 0xF3) | ((val & 0x03) << 2);
        }
        Ok(())
    }

    fn species(&self) -> String {
        let dex = self.nat_dex_number();
        if dex == 0 {
            return String::new();
        }
        BdspGameData.pk_species(dex).unwrap_or_default()
    }

    fn set_species(&mut self, species: &str) -> Result<(), PokemonError> {
        let id = BdspGameData
            .nat_dex_num(species)
            .map_err(|_| PokemonError::UnknownSpecies(species.to_string()))?;
        if let Some(dest) = self.data.get_mut(0x08..0x0A) {
            LittleEndian::write_u16(dest, id);
        }
        // The old form index is meaningless for a different species.
        if let Some(dest) = self.data.get_mut(0x24..0x26) {
            LittleEndian::write_u16(dest, 0);
        }
        self.init_stats();
        Ok(())
    }

    fn ot_name(&self) -> String {
        read_utf16le(&self.data, 0xF8, 26)
    }

    fn set_ot_name(&mut self, ot_name: &str) -> Result<(), PokemonError> {
        if self.data.len() >= 0xF8 + 26 {
            write_utf16le(&mut self.data, 0xF8, ot_name, 12);
        }
        Ok(())
    }

    fn ot_id(&self) -> TrainerID {
        let id32 = LittleEndian::read_u32(self.data.get(0x0C..0x10).unwrap_or(&[0u8; 4]));
        TrainerID {
            public: (id32 & 0xFFFF) as u16,
            private: (id32 >> 16) as u16,
        }
    }

    fn set_ot_id(&mut self, ot_id: &TrainerID) -> Result<(), PokemonError> {
        let id32 = (ot_id.private as u32) << 16 | ot_id.public as u32;
        if let Some(dest) = self.data.get_mut(0x0C..0x10) {
            LittleEndian::write_u32(dest, id32);
        }
        Ok(())
    }

    fn nickname(&self) -> String {
        read_utf16le(&self.data, 0x58, 26)
    }

    fn set_nickname(&mut self, nickname: &str) -> Result<(), PokemonError> {
        if self.data.len() >= 0x58 + 26 {
            write_utf16le(&mut self.data, 0x58, nickname, 12);
        }
        Ok(())
    }

    fn held_item(&self) -> Option<String> {
        let id = LittleEndian::read_u16(self.data.get(0x0A..0x0C).unwrap_or(&[0u8; 2]));
        if id == 0 {
            return None;
        }
        Connection::open("pk_edit.db").ok().and_then(|conn| {
            conn.query_row(
                "SELECT name_en FROM items WHERE id_in_game = ?1 AND game_family = 'bdsp'",
                [id as usize],
                |row| row.get(0),
            )
            .ok()
        })
    }

    fn set_held_item(&mut self, item: &str) -> Result<(), PokemonError> {
        if item == "-" || item == "None" || item.is_empty() {
            if let Some(dest) = self.data.get_mut(0x0A..0x0C) {
                LittleEndian::write_u16(dest, 0);
            }
            return Ok(());
        }
        let id = Connection::open("pk_edit.db")
            .and_then(|conn| {
                conn.query_row(
                    "SELECT id_in_game FROM items WHERE name_en = ?1 AND game_family = 'bdsp'",
                    [item],
                    |row| row.get::<_, u16>(0),
                )
            })
            .map_err(|_| PokemonError::UnknownItem(item.to_string()))?;
        if let Some(dest) = self.data.get_mut(0x0A..0x0C) {
            LittleEndian::write_u16(dest, id);
        }
        Ok(())
    }

    fn moves(&self) -> Vec<Move> {
        let pp_ups = self.data.get(0x7E..0x82).unwrap_or(&[0u8; 4]);
        let pp_list = self.data.get(0x7A..0x7E).unwrap_or(&[0u8; 4]);

        let mut result = Vec::new();
        for i in 0..4usize {
            let move_offset = 0x72 + i * 2;
            let move_id = LittleEndian::read_u16(self.data.get(move_offset..move_offset + 2).unwrap_or(&[0u8; 2]));
            if move_id == 0 {
                continue;
            }
            let bonus = pp_ups.get(i).copied().unwrap_or(0);
            let pp = pp_list.get(i).copied().unwrap_or(0);
            if let Ok((move_type, name, max_pp)) = BdspGameData.move_data(move_id as usize) {
                let boosted = max_pp.saturating_add(max_pp.saturating_mul(bonus) / 5);
                result.push(Move {
                    name,
                    move_type,
                    pp: boosted,
                    pp_used: boosted.saturating_sub(pp),
                });
            }
        }
        result
    }

    fn set_move(&mut self, slot: usize, attack: &str) -> Result<(), PokemonError> {
        if slot > 3 {
            return Err(PokemonError::InvalidMoveSlot(slot));
        }
        let (id, pp) = Connection::open("pk_edit.db")
            .and_then(|conn| {
                conn.query_row(
                    "SELECT id_in_game, pp FROM moves WHERE name_en = ?1 AND game_family = 'bdsp'",
                    [attack],
                    |row| Ok((row.get::<_, u16>(0)?, row.get::<_, u8>(1)?)),
                )
            })
            .map_err(|_| PokemonError::UnknownMove(attack.to_string()))?;

        let move_offset = 0x72 + slot * 2;
        if let Some(dest) = self.data.get_mut(move_offset..move_offset + 2) {
            LittleEndian::write_u16(dest, id);
        }
        if let Some(dest) = self.data.get_mut(0x7A + slot) {
            *dest = pp;
        }
        Ok(())
    }

    fn ability(&self) -> String {
        match BdspGameData.abilities_form(self.nat_dex_number(), self.form()) {
            Ok(abilities) => {
                let ability_num = self.data.get(0x16).copied().unwrap_or(0) & 0x07;
                match ability_num {
                    0 => abilities.slot1,
                    1 => abilities.slot2.unwrap_or(abilities.slot1),
                    2 => abilities.hidden.unwrap_or(abilities.slot1),
                    _ => abilities.slot1,
                }
            }
            Err(_) => String::new(),
        }
    }

    fn set_ability(&mut self, ability: &str) -> Result<(), PokemonError> {
        let dex = self.nat_dex_number();
        let form = self.form();
        let abilities = BdspGameData
            .abilities_form(dex, form)
            .map_err(|_| PokemonError::UnknownSpecies(self.species()))?;

        let num = if abilities.slot1 == ability {
            0u8
        } else if abilities.slot2.as_deref() == Some(ability) {
            1
        } else if abilities.hidden.as_deref() == Some(ability) {
            2
        } else {
            return Ok(());
        };

        if let Some(dest) = self.data.get_mut(0x16) {
            *dest = (*dest & 0xF8) | (num & 0x07);
        }

        let (a1, a2, ah) = BdspGameData
            .ability_ids_form(dex, form)
            .unwrap_or((0, None, None));
        let ability_id = match num {
            0 => a1,
            1 => a2.unwrap_or(a1),
            _ => ah.unwrap_or(a1),
        };
        if let Some(dest) = self.data.get_mut(0x14..0x16) {
            LittleEndian::write_u16(dest, ability_id);
        }
        Ok(())
    }

    fn friendship(&self) -> u8 {
        self.data.get(0x112).copied().unwrap_or(0)
    }

    fn set_friendship(&mut self, value: u8) -> Result<(), PokemonError> {
        if let Some(dest) = self.data.get_mut(0x112) {
            *dest = value;
        }
        Ok(())
    }

    fn pokerus_status(&self) -> Pokerus {
        let byte = self.data.get(0x32).copied().unwrap_or(0);
        let strain = (byte & 0xF0) >> 4;
        let days = byte & 0x0F;
        if strain > 0 && days == 0 {
            Pokerus::Cured
        } else if strain > 0 {
            Pokerus::Infected
        } else {
            Pokerus::None
        }
    }

    fn set_pokerus_status(&mut self, status: Pokerus) -> Result<(), PokemonError> {
        let val = match status {
            Pokerus::None => 0x00u8,
            Pokerus::Infected => 0x11,
            Pokerus::Cured => 0x10,
        };
        if let Some(dest) = self.data.get_mut(0x32) {
            *dest = val;
        }
        Ok(())
    }

    fn ivs(&self) -> StatBlock {
        let iv32 = LittleEndian::read_u32(self.data.get(0x8C..0x90).unwrap_or(&[0u8; 4]));
        StatBlock {
            hp: (iv32 & 0x1F) as u8,
            attack: ((iv32 >> 5) & 0x1F) as u8,
            defense: ((iv32 >> 10) & 0x1F) as u8,
            speed: ((iv32 >> 15) & 0x1F) as u8,
            special_attack: ((iv32 >> 20) & 0x1F) as u8,
            special_defense: ((iv32 >> 25) & 0x1F) as u8,
        }
    }

    fn set_ivs(&mut self, ivs: StatBlock) -> Result<(), PokemonError> {
        let current = LittleEndian::read_u32(self.data.get(0x8C..0x90).unwrap_or(&[0u8; 4]));
        let egg_bit = current & (1 << 30);
        let nick_bit = current & (1 << 31);
        let mut iv32 = egg_bit | nick_bit;
        iv32 |= ivs.hp as u32 & 0x1F;
        iv32 |= (ivs.attack as u32 & 0x1F) << 5;
        iv32 |= (ivs.defense as u32 & 0x1F) << 10;
        iv32 |= (ivs.speed as u32 & 0x1F) << 15;
        iv32 |= (ivs.special_attack as u32 & 0x1F) << 20;
        iv32 |= (ivs.special_defense as u32 & 0x1F) << 25;
        if let Some(dest) = self.data.get_mut(0x8C..0x90) {
            LittleEndian::write_u32(dest, iv32);
        }
        self.init_stats();
        Ok(())
    }

    fn evs(&self) -> StatBlock {
        StatBlock {
            hp: self.data.get(0x26).copied().unwrap_or(0),
            attack: self.data.get(0x27).copied().unwrap_or(0),
            defense: self.data.get(0x28).copied().unwrap_or(0),
            speed: self.data.get(0x29).copied().unwrap_or(0),
            special_attack: self.data.get(0x2A).copied().unwrap_or(0),
            special_defense: self.data.get(0x2B).copied().unwrap_or(0),
        }
    }

    fn set_evs(&mut self, evs: StatBlock) -> Result<(), PokemonError> {
        let vals = [evs.hp, evs.attack, evs.defense, evs.speed, evs.special_attack, evs.special_defense];
        for (i, &val) in vals.iter().enumerate() {
            if let Some(dest) = self.data.get_mut(0x26 + i) {
                *dest = val;
            }
        }
        self.init_stats();
        Ok(())
    }

    fn exp(&self) -> u32 {
        self.experience()
    }

    fn is_egg(&self) -> bool {
        let iv32 = LittleEndian::read_u32(self.data.get(0x8C..0x90).unwrap_or(&[0u8; 4]));
        iv32 & (1 << 30) != 0
    }

    fn is_shiny(&self) -> bool {
        let id = self.ot_id();
        let pid = LittleEndian::read_u32(self.data.get(0x1C..0x20).unwrap_or(&[0u8; 4]));
        let pid_high = (pid >> 16) as u16;
        let pid_low = (pid & 0xFFFF) as u16;
        (id.public ^ id.private ^ pid_high ^ pid_low) < 16
    }

    fn is_bad_egg(&self) -> bool {
        false
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes_encrypted()
    }

    fn is_empty(&self) -> bool {
        self.species_id() == 0
    }

    fn nat_dex_number(&self) -> u16 {
        self.species_id()
    }

    /// Alternate-form index. PB8 stores it as a `u16` at `0x24`; the value fits
    /// in a `u8` for every species in the game.
    fn form(&self) -> u8 {
        LittleEndian::read_u16(self.data.get(0x24..0x26).unwrap_or(&[0u8; 2]))
            .try_into()
            .unwrap_or(0)
    }

    fn personality_value(&self) -> u32 {
        LittleEndian::read_u32(self.data.get(0x1C..0x20).unwrap_or(&[0u8; 4]))
    }

    fn language(&self) -> String {
        "ENG".to_string()
    }

    fn typing(&self) -> Option<(String, Option<String>)> {
        if self.is_empty() {
            return None;
        }
        BdspGameData
            .typing_form(self.nat_dex_number(), self.form())
            .ok()
    }

    fn pokeball_caught(&self) -> usize {
        self.data.get(0x124).copied().unwrap_or(4) as usize
    }

    fn set_pokeball_caught(&mut self, ball_id: u8) -> Result<(), PokemonError> {
        if let Some(dest) = self.data.get_mut(0x124) {
            *dest = ball_id;
        }
        Ok(())
    }

    fn infect_pokerus(&mut self) {
        use rand::Rng;
        let strain = rand::thread_rng().gen_range(1u8..=15);
        let days = (strain % 4) + 1;
        if let Some(dest) = self.data.get_mut(0x32) {
            *dest = (strain << 4) | days;
        }
    }

    fn cure_pokerus(&mut self) {
        if let Some(dest) = self.data.get_mut(0x32) {
            *dest &= 0xF0;
        }
    }

    fn remove_pokerus(&mut self) {
        if let Some(dest) = self.data.get_mut(0x32) {
            *dest = 0;
        }
    }

    fn computed_stats(&self) -> ComputedStats {
        let level = self.level();
        ComputedStats {
            hp: self.stats.hp(level),
            attack: self.stats.attack(level),
            defense: self.stats.defense(level),
            sp_attack: self.stats.sp_attack(level),
            sp_defense: self.stats.sp_defense(level),
            speed: self.stats.speed(level),
        }
    }

    fn update_iv(&mut self, stat: &str, value: u16) {
        self.stats.update_ivs(stat, value);
        let ivs = StatBlock {
            hp: self.stats.hp_iv as u8,
            attack: self.stats.attack_iv as u8,
            defense: self.stats.defense_iv as u8,
            speed: self.stats.speed_iv as u8,
            special_attack: self.stats.sp_attack_iv as u8,
            special_defense: self.stats.sp_defense_iv as u8,
        };
        let _ = self.set_ivs(ivs);
    }

    fn update_ev(&mut self, stat: &str, value: u16) {
        self.stats.update_evs(stat, value);
        let evs = StatBlock {
            hp: self.stats.hp_ev as u8,
            attack: self.stats.attack_ev as u8,
            defense: self.stats.defense_ev as u8,
            speed: self.stats.speed_ev as u8,
            special_attack: self.stats.sp_attack_ev as u8,
            special_defense: self.stats.sp_defense_ev as u8,
        };
        let _ = self.set_evs(evs);
    }

    fn update_checksum(&mut self) {
        let chk = crypto::calculate_checksum(&self.data);
        if let Some(dest) = self.data.get_mut(0x06..0x08) {
            LittleEndian::write_u16(dest, chk);
        }
    }
}

impl BdspPokemon {
    fn update_exp_for_level(&mut self, level: u8) {
        let dex = self.nat_dex_number();
        let growth = BdspGameData.growth_rate(dex).unwrap_or_default();
        let growth_idx = growth_rate_index(&growth);
        let exp = crate::gen3::game_data::EXPERIENCE_TABLE
            .get((level as usize).saturating_sub(1))
            .and_then(|row| row.get(growth_idx))
            .copied()
            .unwrap_or(0);
        if let Some(dest) = self.data.get_mut(0x10..0x14) {
            LittleEndian::write_u32(dest, exp);
        }
    }
}

fn growth_rate_index(growth: &str) -> usize {
    match growth {
        "Erratic" => 0,
        "Fast" => 1,
        "Medium Fast" => 2,
        "Medium Slow" => 3,
        "Slow" => 4,
        "Fluctuating" => 5,
        _ => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A freshly created Pokémon must have its party stat block and current HP
    /// written, otherwise the game reads it as fainted (current HP 0) with a
    /// level-0 stat line. This exercises the byte layout without touching the DB.
    fn read_u16_at(mon: &BdspPokemon, offset: usize) -> u16 {
        LittleEndian::read_u16(mon.data.get(offset..offset + 2).unwrap_or(&[0u8; 2]))
    }

    #[test]
    fn recompute_party_stats_writes_current_hp_and_block() {
        let mut mon = BdspPokemon::new();
        // Mark non-empty (species id != 0) and set level directly at the raw
        // offsets so we don't need the species DB.
        if let Some(dest) = mon.data.get_mut(0x08..0x0A) {
            LittleEndian::write_u16(dest, 1);
        }
        let level = 50u8;
        if let Some(dest) = mon.data.get_mut(0x148) {
            *dest = level;
        }
        // Give it deterministic base stats via the cached Stats struct.
        mon.stats = Stats {
            base_hp: 100,
            base_attack: 80,
            base_defense: 70,
            base_sp_attack: 60,
            base_sp_defense: 65,
            base_speed: 90,
            n_mod: crate::gen3::game_data::NATURE_MODIFIER[0],
            ..Stats::default()
        };

        mon.recompute_party_stats();

        let expected_hp = mon.stats.hp(level);
        assert!(expected_hp > 0);
        assert_eq!(
            read_u16_at(&mon, 0x8A),
            expected_hp,
            "current HP must be full, not 0 (fainted)"
        );
        assert_eq!(
            read_u16_at(&mon, 0x14A),
            expected_hp,
            "max HP must be written to party block"
        );
        assert_eq!(
            mon.data.get(0x148).copied(),
            Some(level),
            "party-block level must match"
        );
        assert_eq!(read_u16_at(&mon, 0x14C), mon.stats.attack(level));
    }

    /// The party block must stay zeroed for an empty slot so blank slots don't
    /// gain a phantom stat line.
    #[test]
    fn recompute_party_stats_skips_empty() {
        let mut mon = BdspPokemon::new();
        mon.recompute_party_stats();
        assert_eq!(read_u16_at(&mon, 0x8A), 0);
        assert_eq!(read_u16_at(&mon, 0x14A), 0);
    }

    /// The form index lives at `0x24` and must round-trip through the raw
    /// bytes, since that is what the game and the sprite lookup both read.
    #[test]
    fn form_round_trips_through_offset_0x24() {
        let mut mon = BdspPokemon::new();
        assert_eq!(mon.form(), 0, "a fresh record defaults to the base form");

        mon.set_form(5);
        assert_eq!(read_u16_at(&mon, 0x24), 5, "form must be written to 0x24");
        assert_eq!(mon.form(), 5);

        mon.set_form(0);
        assert_eq!(mon.form(), 0);
    }
}
