//! The core [`Gen3Pokemon`] struct and its full API.
//!
//! [`Gen3Pokemon`] is a 100-byte (party) or 80-byte (PC) structure. The first 80 bytes are always
//! present; the last 20 bytes (status condition, level mirror, current/total HP, and combat stats)
//! are only stored for party Pokémon. [`Gen3Pokemon::from_bytes`] handles both sizes automatically.
//!
//! Internally, bytes 0x20–0x4F hold four 12-byte sub-structures (see [`crate::gen3::pokemon::data`])
//! that are XOR-encrypted with `PID ^ OT_ID` and shuffled according to `PID % 24`.

use crate::common::charset::gen3::{get_char, get_code};
use crate::common::types::{ComputedStats, Gender, Move, Pokerus, StatBlock, TrainerID};
use crate::error::PokemonError;
use crate::gen3::game_data::{Gen3GameData, EXPERIENCE_TABLE, NATURE, SPECIES};
use crate::gen3::pokemon::crypto::{calculate_checksum, crypt_data};
use crate::gen3::pokemon::data::*;
use crate::gen3::pokemon::factory::generate_method_1;
use crate::traits::game_data::GameData;
use crate::traits::pokemon::Pokemon;
use byteorder::{ByteOrder, LittleEndian};
use rand::Rng;
use rusqlite::Connection;

/// A parsed Pokémon from a Generation III save file.
///
/// Encapsulates both the persistent 80-byte data (always present in PC and party storage)
/// and the volatile 20-byte party data (only meaningful when the Pokémon is in the party).
/// Use [`Gen3Pokemon::from_bytes`] to parse raw bytes and [`Gen3Pokemon::to_bytes_array`] to serialise back.
#[derive(Debug, Default, Copy, Clone)]
pub struct Gen3Pokemon {
    /// Byte offset of this Pokémon within the save file buffer.
    pub offset: usize,
    /// Personality value (PID). Determines gender, nature, ability, shiny status, and block order.
    pub personality_value: u32,
    /// Original trainer ID bytes (little-endian: public u16, then private u16).
    ot_id: [u8; 4],
    /// Nickname encoded in Gen III character set (10 bytes, space-padded).
    nickname: [u8; 10],
    /// Language code byte (see [`Language`]).
    language: u8,
    /// Miscellaneous flags byte (bit 0 = bad egg, etc.).
    misc_flags: u8,
    /// OT name encoded in Gen III character set (7 bytes, 0xFF-terminated).
    ot_name: [u8; 7],
    /// Pokémon markings byte (circle/square/triangle/heart flags).
    markings: u8,
    /// 16-bit checksum of the unencrypted sub-structure data.
    pub checksum: u16,
    /// Unused padding bytes between header and encrypted data.
    padding: u16,
    /// The four decrypted and de-shuffled sub-structures.
    pub(crate) data: PokemonData,
    /// Status condition bitfield (poisoned, burned, frozen, etc.). Party-only.
    status_condition: u32,
    /// Level mirror stored in party data (recalculated from experience on load). Party-only.
    level: u8,
    /// Mail ID attached to the Pokémon (0xFF = none). Party-only.
    mail_id: u8,
    /// Current HP remaining. Party-only.
    current_hp: u16,
    /// Maximum HP. Party-only.
    total_hp: u16,
    /// Calculated battle stats (Attack, Defense, Speed, Sp. Atk, Sp. Def).
    pub stats: Stats,
}

// --- Gen3Pokemon inherent methods ---

impl Gen3Pokemon {
    /// Parses a [`Gen3Pokemon`] from a byte slice.
    ///
    /// Accepts both 80-byte (PC format) and 100-byte (party format) slices.
    ///
    /// # Errors
    /// Returns [`PokemonError::InvalidDataLength`] if the slice is shorter than 80 bytes.
    pub fn from_bytes(offset: usize, buffer: &[u8]) -> Result<Self, PokemonError> {
        if buffer.len() < 80 {
            return Err(PokemonError::InvalidDataLength(buffer.len()));
        }

        let personality_value = LittleEndian::read_u32(&buffer[0x00..0x04]);
        let ot_id_u32 = LittleEndian::read_u32(&buffer[0x04..0x08]);
        let key = personality_value ^ ot_id_u32;

        let mut raw_data = [0u8; 48];
        raw_data.copy_from_slice(&buffer[0x20..0x50]);
        crypt_data(&mut raw_data, key);

        let pid_mod = (personality_value % 24) as usize;
        let order = BLOCK_ORDERS[pid_mod];

        let mut growth = GrowthBlock::default();
        let mut attacks = AttacksBlock::default();
        let mut evs = EVsBlock::default();
        let mut misc = MiscBlock::default();

        for (i, block_type) in order.iter().enumerate() {
            let start = i * 12;
            let chunk = &raw_data[start..start + 12];

            match block_type {
                BlockType::Growth => {
                    growth.species = LittleEndian::read_u16(&chunk[0..2]);
                    growth.item = LittleEndian::read_u16(&chunk[2..4]);
                    growth.experience = LittleEndian::read_u32(&chunk[4..8]);
                    growth.pp_bonuses = chunk[8];
                    growth.friendship = chunk[9];
                    growth.unknown = LittleEndian::read_u16(&chunk[10..12]);
                }
                BlockType::Attacks => {
                    growth_slice_to_u16_array(&chunk[0..8], &mut attacks.moves);
                    attacks.pp.copy_from_slice(&chunk[8..12]);
                }
                BlockType::EVs => {
                    evs.hp = chunk[0];
                    evs.attack = chunk[1];
                    evs.defense = chunk[2];
                    evs.speed = chunk[3];
                    evs.sp_attack = chunk[4];
                    evs.sp_defense = chunk[5];
                    evs.coolness = chunk[6];
                    evs.beauty = chunk[7];
                    evs.cuteness = chunk[8];
                    evs.smartness = chunk[9];
                    evs.toughness = chunk[10];
                    evs.feel = chunk[11];
                }
                BlockType::Misc => {
                    misc.pokerus = chunk[0];
                    misc.met_location = chunk[1];
                    misc.origins_info = OriginsInfo::from_bytes([chunk[2], chunk[3]]);
                    misc.iv_egg_ability =
                        IVsEggAbility::from_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
                    misc.ribbons_obedience =
                        RibbonsObedience::from_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]);
                }
            }
        }

        let mut pokemon = Gen3Pokemon {
            offset,
            personality_value,
            ot_id: buffer[0x04..0x08].try_into().unwrap_or_default(),
            nickname: buffer[0x08..0x12].try_into().unwrap_or_default(),
            language: buffer[0x12],
            misc_flags: buffer[0x13],
            ot_name: buffer[0x14..0x1B].try_into().unwrap_or_default(),
            markings: buffer[0x1B],
            checksum: LittleEndian::read_u16(&buffer[0x1C..0x1E]),
            padding: LittleEndian::read_u16(&buffer[0x1E..0x20]),
            data: PokemonData {
                growth,
                attacks,
                evs,
                misc,
            },
            ..Default::default()
        };

        pokemon.init_stats();

        if buffer.len() >= 100 {
            pokemon.status_condition = LittleEndian::read_u32(&buffer[0x50..0x54]);
            pokemon.level = buffer[0x54];
            pokemon.mail_id = buffer[0x55];
            pokemon.current_hp = LittleEndian::read_u16(&buffer[0x56..0x58]);
            pokemon.total_hp = LittleEndian::read_u16(&buffer[0x58..0x5A]);
        } else {
            pokemon.current_hp = pokemon.stats.hp(pokemon.level);
            pokemon.total_hp = pokemon.stats.hp(pokemon.level);
            pokemon.status_condition = 0;
        }

        Ok(pokemon)
    }

    /// Serialises into a 100-byte array (party format).
    ///
    /// When saving to PC storage, the caller should write only the first 80 bytes.
    pub fn to_bytes_array(&self) -> [u8; 100] {
        let mut buffer = [0u8; 100];

        LittleEndian::write_u32(&mut buffer[0x00..0x04], self.personality_value);
        buffer[0x04..0x08].copy_from_slice(&self.ot_id);
        buffer[0x08..0x12].copy_from_slice(&self.nickname);
        buffer[0x12] = self.language;
        buffer[0x13] = self.misc_flags;
        buffer[0x14..0x1B].copy_from_slice(&self.ot_name);
        buffer[0x1B] = self.markings;
        LittleEndian::write_u16(&mut buffer[0x1E..0x20], self.padding);

        let mut flat_data = [0u8; 48];
        let g = &self.data.growth;
        let a = &self.data.attacks;
        let e = &self.data.evs;
        let m = &self.data.misc;

        LittleEndian::write_u16(&mut flat_data[0..2], g.species);
        LittleEndian::write_u16(&mut flat_data[2..4], g.item);
        LittleEndian::write_u32(&mut flat_data[4..8], g.experience);
        flat_data[8] = g.pp_bonuses;
        flat_data[9] = g.friendship;
        LittleEndian::write_u16(&mut flat_data[10..12], g.unknown);

        for (i, move_id) in a.moves.iter().enumerate() {
            LittleEndian::write_u16(&mut flat_data[12 + (i * 2)..14 + (i * 2)], *move_id);
        }
        flat_data[20..24].copy_from_slice(&a.pp);

        flat_data[24..36].copy_from_slice(&[
            e.hp,
            e.attack,
            e.defense,
            e.speed,
            e.sp_attack,
            e.sp_defense,
            e.coolness,
            e.beauty,
            e.cuteness,
            e.smartness,
            e.toughness,
            e.feel,
        ]);

        flat_data[36] = m.pokerus;
        flat_data[37] = m.met_location;
        flat_data[38..40].copy_from_slice(&m.origins_info.into_bytes());
        flat_data[40..44].copy_from_slice(&m.iv_egg_ability.into_bytes());
        flat_data[44..48].copy_from_slice(&m.ribbons_obedience.into_bytes());

        let checksum = calculate_checksum(&flat_data);
        LittleEndian::write_u16(&mut buffer[0x1C..0x1E], checksum);

        let pid_mod = (self.personality_value % 24) as usize;
        let order = BLOCK_ORDERS[pid_mod];
        let mut shuffled_data = [0u8; 48];

        for (i, block_type) in order.iter().enumerate() {
            let src_range = match block_type {
                BlockType::Growth => 0..12,
                BlockType::Attacks => 12..24,
                BlockType::EVs => 24..36,
                BlockType::Misc => 36..48,
            };
            shuffled_data[i * 12..i * 12 + 12].copy_from_slice(&flat_data[src_range]);
        }

        let ot_id_u32 = LittleEndian::read_u32(&self.ot_id);
        let key = self.personality_value ^ ot_id_u32;
        crypt_data(&mut shuffled_data, key);
        buffer[0x20..0x50].copy_from_slice(&shuffled_data);

        LittleEndian::write_u32(&mut buffer[0x50..0x54], self.status_condition);
        buffer[0x54] = self.level;
        buffer[0x55] = self.mail_id;
        LittleEndian::write_u16(&mut buffer[0x56..0x58], self.current_hp);
        LittleEndian::write_u16(&mut buffer[0x58..0x5A], self.total_hp);
        LittleEndian::write_u16(&mut buffer[0x5A..0x5C], self.stats.attack);
        LittleEndian::write_u16(&mut buffer[0x5C..0x5E], self.stats.defense);
        LittleEndian::write_u16(&mut buffer[0x5E..0x60], self.stats.speed);
        LittleEndian::write_u16(&mut buffer[0x60..0x62], self.stats.sp_attack);
        LittleEndian::write_u16(&mut buffer[0x62..0x64], self.stats.sp_defense);

        buffer
    }

    /// Returns the raw Gen III species ID stored in the save file.
    pub fn species_id(&self) -> u16 {
        self.data.growth.species
    }

    /// Returns the total accumulated experience points.
    pub fn experience(&self) -> u32 {
        self.data.growth.experience
    }

    /// Returns a mutable reference to the calculated stats.
    pub fn stats_mut(&mut self) -> &mut Stats {
        &mut self.stats
    }

    /// Sets the 1-byte language code (e.g. 2 = English).
    pub fn set_language(&mut self, language: u8) {
        self.language = language;
    }

    // --- Private helpers ---

    fn nature_index(&self) -> usize {
        (self.personality_value % 25) as usize
    }

    fn ability_index(&self) -> usize {
        self.data.misc.iv_egg_ability.ability_bit() as usize
    }

    fn calculate_level_from_exp(&self) -> u8 {
        if self.is_empty() {
            return 0;
        }
        let growth = Gen3GameData
            .growth_rate(self.nat_dex_number())
            .unwrap_or_default();
        find_level(self.experience(), growth_index(&growth)) as u8
    }
}

// --- Pokemon trait implementation ---

impl Pokemon for Gen3Pokemon {
    fn level(&self) -> u8 {
        self.calculate_level_from_exp()
    }

    fn set_level(&mut self, level: u8) -> Result<(), PokemonError> {
        let level = level.clamp(1, 100);
        let growth = Gen3GameData
            .growth_rate(self.nat_dex_number())
            .unwrap_or_default();
        self.data.growth.experience = EXPERIENCE_TABLE[(level - 1) as usize][growth_index(&growth)];
        self.init_stats();
        Ok(())
    }

    fn nature(&self) -> String {
        if self.is_empty() {
            return String::new();
        }
        NATURE[self.nature_index()].to_string()
    }

    fn set_nature(&mut self, nature: &str) -> Result<(), PokemonError> {
        loop {
            let (pid, ivs) = generate_method_1(None);
            if nature == NATURE[(pid % 25) as usize]
                && self.gender() == gender_from_p(pid, self.nat_dex_number())
                && self.ability_index() == ivs.ability_bit() as usize
            {
                self.personality_value = pid;
                self.data.misc.iv_egg_ability = ivs;
                self.init_stats();
                self.update_checksum();
                break;
            }
        }
        Ok(())
    }

    fn gender(&self) -> Gender {
        gender_from_p(self.personality_value, self.nat_dex_number())
    }

    fn set_gender(&mut self, gender: Gender) -> Result<(), PokemonError> {
        let current_nature = self.nature();
        loop {
            let (pid, ivs) = generate_method_1(None);
            if gender_from_p(pid, self.nat_dex_number()) == gender
                && NATURE[(pid % 25) as usize] == current_nature.as_str()
                && ivs.ability_bit() as usize == self.ability_index()
            {
                self.personality_value = pid;
                self.data.misc.iv_egg_ability = ivs;
                self.init_stats();
                self.update_checksum();
                break;
            }
        }
        Ok(())
    }

    fn species(&self) -> String {
        let dex_num = self.nat_dex_number();
        if dex_num == 0 {
            return String::new();
        }
        Gen3GameData.pk_species(dex_num).unwrap_or_default()
    }

    fn set_species(&mut self, species: &str) -> Result<(), PokemonError> {
        if self.species().to_uppercase() == self.nickname() {
            self.set_nickname(&species.to_uppercase())?;
        }

        let mut id = Gen3GameData
            .nat_dex_num(species)
            .map_err(|_| PokemonError::UnknownSpecies(species.to_string()))?;

        id = if id == 0 {
            412
        } else if id >= 252 {
            SPECIES[(id as usize).saturating_sub(251)]
        } else {
            id
        };

        self.data.growth.species = id;

        let min = Gen3GameData.lowest_level(self.nat_dex_number());
        if min > self.level() {
            self.set_level(min)?;
        }

        Ok(())
    }

    fn ot_name(&self) -> String {
        self.ot_name
            .iter()
            .map(|&c| get_char(c as usize))
            .collect::<Vec<&str>>()
            .join("")
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string()
    }

    fn set_ot_name(&mut self, ot_name: &str) -> Result<(), PokemonError> {
        let mut encoded = [0xFF_u8; 7];
        for (i, c) in ot_name.chars().take(7).enumerate() {
            encoded[i] = get_code(&c.to_string());
        }
        self.ot_name.copy_from_slice(&encoded);
        Ok(())
    }

    fn ot_id(&self) -> TrainerID {
        TrainerID {
            public: LittleEndian::read_u16(&self.ot_id[0..2]),
            private: LittleEndian::read_u16(&self.ot_id[2..4]),
        }
    }

    fn set_ot_id(&mut self, ot_id: &TrainerID) -> Result<(), PokemonError> {
        LittleEndian::write_u16(&mut self.ot_id[0..2], ot_id.public);
        LittleEndian::write_u16(&mut self.ot_id[2..4], ot_id.private);
        Ok(())
    }

    fn nickname(&self) -> String {
        self.nickname
            .iter()
            .map(|&c| get_char(c as usize))
            .collect::<Vec<&str>>()
            .join("")
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string()
    }

    fn set_nickname(&mut self, nickname: &str) -> Result<(), PokemonError> {
        let encoded: Vec<u8> = format!("{: <10}", nickname)
            .chars()
            .map(|c| get_code(&c.to_string()))
            .collect();
        self.nickname.copy_from_slice(&encoded);
        Ok(())
    }

    fn held_item(&self) -> Option<String> {
        let id = self.data.growth.item as usize;
        if id == 0 {
            return None;
        }
        Connection::open("pk_edit.db").ok().and_then(|conn| {
            conn.query_row(
                "SELECT name_en FROM items WHERE id_in_game = ?1 AND game_family = 'gen3'",
                [id],
                |row| row.get(0),
            )
            .ok()
        })
    }

    fn set_held_item(&mut self, item: &str) -> Result<(), PokemonError> {
        if item == "-" || item == "None" || item.is_empty() {
            self.data.growth.item = 0;
            return Ok(());
        }
        let id = Connection::open("pk_edit.db")
            .and_then(|conn| {
                conn.query_row(
                    "SELECT id_in_game FROM items WHERE name_en = ?1 AND game_family = 'gen3'",
                    [item],
                    |row| row.get::<_, u16>(0),
                )
            })
            .map_err(|_| PokemonError::UnknownItem(item.to_string()))?;
        self.data.growth.item = id;
        Ok(())
    }

    fn moves(&self) -> Vec<Move> {
        let pp_bonuses = self.data.growth.pp_bonuses;
        self.data
            .attacks
            .moves
            .iter()
            .zip(self.data.attacks.pp.iter())
            .enumerate()
            .filter(|(_, (&id, _))| id != 0)
            .filter_map(|(slot, (&id, &pp_remaining))| {
                let bonus = u8::from((pp_bonuses >> (slot * 2)) & 0x03);
                Gen3GameData
                    .move_data(id as usize)
                    .ok()
                    .map(|(move_type, name, max_pp)| {
                        let boosted = max_pp.saturating_add(max_pp.saturating_mul(bonus) / 5);
                        Move {
                            name,
                            move_type,
                            pp: boosted,
                            pp_used: boosted.saturating_sub(pp_remaining),
                        }
                    })
            })
            .collect()
    }

    fn set_move(&mut self, slot: usize, attack: &str) -> Result<(), PokemonError> {
        if slot > 3 {
            return Err(PokemonError::InvalidMoveSlot(slot));
        }
        let (id, pp) = Connection::open("pk_edit.db")
            .and_then(|conn| {
                conn.query_row(
                    "SELECT id_in_game, pp FROM moves WHERE name_en = ?1 AND game_family = 'gen3'",
                    [attack],
                    |row| Ok((row.get::<_, u16>(0)?, row.get::<_, u8>(1)?)),
                )
            })
            .map_err(|_| PokemonError::UnknownMove(attack.to_string()))?;
        self.data.attacks.moves[slot] = id;
        self.data.attacks.pp[slot] = pp;
        Ok(())
    }

    fn ability(&self) -> String {
        match Gen3GameData.abilities(self.nat_dex_number()) {
            Ok(abilities) => {
                if self.ability_index() == 0 {
                    abilities.slot1
                } else {
                    abilities.slot2.unwrap_or(abilities.slot1)
                }
            }
            Err(_) => String::new(),
        }
    }

    fn set_ability(&mut self, ability: &str) -> Result<(), PokemonError> {
        let abilities = Gen3GameData
            .abilities(self.nat_dex_number())
            .map_err(|_| PokemonError::UnknownSpecies(self.species()))?;

        if abilities.slot1 == ability {
            self.data.misc.iv_egg_ability.set_ability_bit(0);
        } else if abilities.slot2.as_deref() == Some(ability) {
            self.data.misc.iv_egg_ability.set_ability_bit(1);
        }
        Ok(())
    }

    fn friendship(&self) -> u8 {
        self.data.growth.friendship
    }

    fn set_friendship(&mut self, value: u8) -> Result<(), PokemonError> {
        self.data.growth.friendship = value;
        Ok(())
    }

    fn pokerus_status(&self) -> Pokerus {
        let strain = (self.data.misc.pokerus & 0xF0) >> 4;
        let days = self.data.misc.pokerus & 0x0F;
        if strain > 0 && days == 0 {
            Pokerus::Cured
        } else if strain > 0 {
            Pokerus::Infected
        } else {
            Pokerus::None
        }
    }

    fn set_pokerus_status(&mut self, status: Pokerus) -> Result<(), PokemonError> {
        self.data.misc.pokerus = match status {
            Pokerus::None => 0x00,
            Pokerus::Infected => 0x11, // strain 1, 1 day remaining
            Pokerus::Cured => 0x10,    // strain 1, 0 days remaining
        };
        Ok(())
    }

    fn ivs(&self) -> StatBlock {
        let iv = self.data.misc.iv_egg_ability;
        StatBlock {
            hp: iv.hp_iv(),
            attack: iv.attack_iv(),
            defense: iv.defense_iv(),
            special_attack: iv.sp_attack_iv(),
            special_defense: iv.sp_defense_iv(),
            speed: iv.speed_iv(),
        }
    }

    fn set_ivs(&mut self, ivs: StatBlock) -> Result<(), PokemonError> {
        let iv = &mut self.data.misc.iv_egg_ability;
        iv.set_hp_iv(ivs.hp);
        iv.set_attack_iv(ivs.attack);
        iv.set_defense_iv(ivs.defense);
        iv.set_sp_attack_iv(ivs.special_attack);
        iv.set_sp_defense_iv(ivs.special_defense);
        iv.set_speed_iv(ivs.speed);
        self.init_stats();
        Ok(())
    }

    fn evs(&self) -> StatBlock {
        let e = &self.data.evs;
        StatBlock {
            hp: e.hp,
            attack: e.attack,
            defense: e.defense,
            special_attack: e.sp_attack,
            special_defense: e.sp_defense,
            speed: e.speed,
        }
    }

    fn set_evs(&mut self, evs: StatBlock) -> Result<(), PokemonError> {
        let e = &mut self.data.evs;
        e.hp = evs.hp;
        e.attack = evs.attack;
        e.defense = evs.defense;
        e.sp_attack = evs.special_attack;
        e.sp_defense = evs.special_defense;
        e.speed = evs.speed;
        self.init_stats();
        Ok(())
    }

    fn exp(&self) -> u32 {
        self.data.growth.experience
    }

    fn is_egg(&self) -> bool {
        self.data.misc.iv_egg_ability.is_egg() != 0
    }

    fn is_shiny(&self) -> bool {
        let public = LittleEndian::read_u16(&self.ot_id[0..2]);
        let private = LittleEndian::read_u16(&self.ot_id[2..4]);
        let pid_high = (self.personality_value >> 16) as u16;
        let pid_low = (self.personality_value & 0xFFFF) as u16;
        (public ^ private ^ pid_high ^ pid_low) < 8
    }

    fn is_bad_egg(&self) -> bool {
        self.misc_flags & 0b0000_0001 == 1
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes_array().to_vec()
    }

    fn is_empty(&self) -> bool {
        self.personality_value == 0
    }

    fn nat_dex_number(&self) -> u16 {
        let species = self.species_id();
        if species == 412 {
            return 0;
        }
        if species >= 277 {
            return SPECIES
                .iter()
                .position(|&x| x == species)
                .unwrap_or(0)
                .saturating_add(251)
                .try_into()
                .unwrap_or(0);
        }
        species
    }

    /// Alternate-form index.
    ///
    /// Generation III has no form byte. Unown is the only species with stored
    /// forms, and its letter is derived from the PID by concatenating the low
    /// two bits of each PID byte, mod 28. Every other species reports 0.
    ///
    /// Read-only: changing the form would require regenerating the PID.
    fn form(&self) -> u8 {
        const UNOWN_DEX: u16 = 201;
        if self.nat_dex_number() != UNOWN_DEX {
            return 0;
        }
        let pid = self.personality_value;
        let value = ((pid & 0x0300_0000) >> 18)
            | ((pid & 0x0003_0000) >> 12)
            | ((pid & 0x0000_0300) >> 6)
            | (pid & 0x0000_0003);
        (value % 28) as u8
    }

    fn personality_value(&self) -> u32 {
        self.personality_value
    }

    fn language(&self) -> String {
        Language::from(self.language).to_string()
    }

    fn typing(&self) -> Option<(String, Option<String>)> {
        if self.is_empty() {
            return None;
        }
        Gen3GameData.typing(self.nat_dex_number()).ok()
    }

    fn pokeball_caught(&self) -> usize {
        self.data.misc.origins_info.pokeball() as usize
    }

    fn set_pokeball_caught(&mut self, ball_id: u8) -> Result<(), PokemonError> {
        if ball_id > 12 {
            return Err(PokemonError::InvalidPokeball(ball_id));
        }
        self.data.misc.origins_info.set_pokeball(ball_id);
        Ok(())
    }

    fn infect_pokerus(&mut self) {
        let mut rng = rand::thread_rng();
        let strain = rng.gen_range(1u8..=15);
        let days = (strain % 4) + 1;
        self.data.misc.pokerus = (strain << 4) | days;
    }

    fn cure_pokerus(&mut self) {
        self.data.misc.pokerus &= 0xF0;
    }

    fn remove_pokerus(&mut self) {
        self.data.misc.pokerus = 0;
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
        let iv = &mut self.data.misc.iv_egg_ability;
        match stat {
            "HP" => iv.set_hp_iv(self.stats.hp_iv as u8),
            "Attack" => iv.set_attack_iv(self.stats.attack_iv as u8),
            "Defense" => iv.set_defense_iv(self.stats.defense_iv as u8),
            "Sp. Atk" => iv.set_sp_attack_iv(self.stats.sp_attack_iv as u8),
            "Sp. Def" => iv.set_sp_defense_iv(self.stats.sp_defense_iv as u8),
            "Speed" => iv.set_speed_iv(self.stats.speed_iv as u8),
            _ => {}
        }
    }

    fn update_ev(&mut self, stat: &str, value: u16) {
        self.stats.update_evs(stat, value);
        let e = &mut self.data.evs;
        match stat {
            "HP" => e.hp = self.stats.hp_ev as u8,
            "Attack" => e.attack = self.stats.attack_ev as u8,
            "Defense" => e.defense = self.stats.defense_ev as u8,
            "Sp. Atk" => e.sp_attack = self.stats.sp_attack_ev as u8,
            "Sp. Def" => e.sp_defense = self.stats.sp_defense_ev as u8,
            "Speed" => e.speed = self.stats.speed_ev as u8,
            _ => {}
        }
    }

    fn update_checksum(&mut self) {
        let mut flat_data = [0u8; 48];
        let g = &self.data.growth;
        let a = &self.data.attacks;
        let e = &self.data.evs;
        let m = &self.data.misc;

        LittleEndian::write_u16(&mut flat_data[0..2], g.species);
        LittleEndian::write_u16(&mut flat_data[2..4], g.item);
        LittleEndian::write_u32(&mut flat_data[4..8], g.experience);
        flat_data[8] = g.pp_bonuses;
        flat_data[9] = g.friendship;
        LittleEndian::write_u16(&mut flat_data[10..12], g.unknown);

        for (i, move_id) in a.moves.iter().enumerate() {
            LittleEndian::write_u16(&mut flat_data[12 + (i * 2)..14 + (i * 2)], *move_id);
        }
        flat_data[20..24].copy_from_slice(&a.pp);

        flat_data[24..36].copy_from_slice(&[
            e.hp,
            e.attack,
            e.defense,
            e.speed,
            e.sp_attack,
            e.sp_defense,
            e.coolness,
            e.beauty,
            e.cuteness,
            e.smartness,
            e.toughness,
            e.feel,
        ]);

        flat_data[36] = m.pokerus;
        flat_data[37] = m.met_location;
        flat_data[38..40].copy_from_slice(&m.origins_info.into_bytes());
        flat_data[40..44].copy_from_slice(&m.iv_egg_ability.into_bytes());
        flat_data[44..48].copy_from_slice(&m.ribbons_obedience.into_bytes());

        self.checksum = calculate_checksum(&flat_data);
    }
}

// --- Private free functions ---

fn gender_from_p(pid: u32, dex_num: u16) -> Gender {
    let ratio = match Gen3GameData.gender_ratio(dex_num) {
        Ok(r) => r,
        Err(_) => return Gender::None,
    };
    match ratio {
        255 => Gender::None,
        254 => Gender::F,
        0 => Gender::M,
        threshold => {
            if (pid % 256) < threshold as u32 {
                Gender::F
            } else {
                Gender::M
            }
        }
    }
}
