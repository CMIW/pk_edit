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
use byteorder::{ByteOrder, LittleEndian};
use rand::Rng;
use serde_json::Value;
use std::fmt;
use thiserror::Error;

use crate::data_structure::character_set::{get_char, get_code, CharacterSet};
use crate::data_structure::save_data::TrainerID;
use crate::misc::{
    ability, find_item, gender_ratio, growth_rate, hidden_ability, item_id_g3, move_data,
    nat_dex_num, pk_species, typing, EXPERIENCE_TABLE, GENDER_THRESHOLD, MOVES, NATURE,
    NATURE_MODIFIER, POKEDEX_JSON, SPECIES,
};

/// Errors related to Pokémon data handling.
#[derive(Error, Debug)]
pub enum PokemonError {
    #[error("Invalid data length: expected 48 bytes, found {0}")]
    InvalidDataLength(usize),

    #[error("Species '{0}' not recognized")]
    UnknownSpecies(String),

    #[error("Gender ratio data missing for dex number {0}")]
    MissingGenderRatio(u16),
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Pokemon {
    offset: usize,
    personality_value: [u8; 4],
    ot_id: [u8; 4],
    nickname: [u8; 10],
    language: [u8; 1],
    misc_flags: [u8; 1],
    ot_name: [u8; 7],
    markings: [u8; 1],
    checksum: [u8; 2],
    _padding: [u8; 2],
    pokemon_data: PokemonData,
    stats: Stats,
}

impl fmt::Display for Pokemon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "OT: {}\n\
            Level: {}\n\
            PID: {}\n\
            Species: {}\n\
            Nickname: {}\n\
            ",
            self.ot_name(),
            self.level(),
            self.personality_value(),
            self.species(),
            self.nickname(),
        )
    }
}

impl Pokemon {
    pub fn new(offset: usize, buffer: &[u8]) -> Self {
        let pokemon_data = &buffer[0x20..0x50];

        // unencrypt the pokemon data substructure
        // first we obtain the ecryption key by XORing the entire original trainer id with the pokemon personality value
        let ecryption_key = LittleEndian::read_u32(&buffer[0x04..0x08])
            ^ LittleEndian::read_u32(&buffer[0x00..0x04]);

        let mut data: [u8; 48] = [0; 48];
        pokemon_data_encryption(ecryption_key, pokemon_data, &mut data);

        // This section is stored in a special and encrypted format.
        // The order of the structures is determined by the personality value of the Pokémon modulo 24
        let personality_value_modulo = LittleEndian::read_u32(&buffer[0x00..0x04]) % 24;
        let mut pokemon_data = PokemonData::new(data);

        order_data_substructure(personality_value_modulo, &mut pokemon_data);

        let mut personality_value = [0; 4];
        let mut ot_id = [0; 4];
        let mut nickname = [0; 10];
        let mut language = [0];
        let mut misc_flags = [0];
        let mut ot_name = [0; 7];
        let mut markings = [0];
        let mut checksum = [0; 2];
        let mut _padding = [0; 2];

        personality_value.copy_from_slice(&buffer[0x00..0x04]);
        ot_id.copy_from_slice(&buffer[0x04..0x08]);
        nickname.copy_from_slice(&buffer[0x08..0x12]);
        language.copy_from_slice(&buffer[0x12..0x13]);
        misc_flags.copy_from_slice(&buffer[0x13..0x14]);
        ot_name.copy_from_slice(&buffer[0x14..0x1B]);
        markings.copy_from_slice(&buffer[0x1B..0x1C]);
        checksum.copy_from_slice(&buffer[0x1C..0x1E]);
        _padding.copy_from_slice(&buffer[0x1E..0x20]);

        let mut pokemon = Pokemon {
            offset,
            personality_value,
            ot_id,
            nickname,
            language,
            misc_flags,
            ot_name,
            markings,
            checksum,
            _padding,
            pokemon_data,
            stats: Stats::default(),
        };

        pokemon.init_stats();

        pokemon
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn ot_id(&self) -> TrainerID {
        self.ot_id.into()
    }

    fn set_ot_id(&mut self, ot_id: &[u8]) {
        self.ot_id.copy_from_slice(ot_id);
    }

    pub fn is_bad_egg(&self) -> bool {
        let flag = self.misc_flags[0] & 0b00000001;

        flag == 1
    }

    pub fn nickname(&self) -> String {
        //let char_set = CharacterSet::new();
        let nickname = &self
            .nickname
            .iter()
            .map(|c| get_char(*c as usize))
            .collect::<Vec<&str>>();

        let nickname = nickname.join("");
        let nickname = nickname.split(' ').next().unwrap();

        nickname.to_string()
    }

    fn set_nickname(&mut self, nickname: &str) {
        let name: Vec<u8> = format!("{: <10}", nickname)
            .chars()
            .map(|s| get_code(&s.to_string()))
            .collect();
        self.nickname.copy_from_slice(&name);
    }

    pub fn language(&self) -> Language {
        self.language.into()
    }

    pub fn ot_name(&self) -> String {
        let ot_name = &self
            .ot_name
            .iter()
            .map(|c| get_char(*c as usize))
            .collect::<Vec<&str>>();

        let ot_name = ot_name.join("");
        let ot_name = ot_name.split(' ').next().unwrap();

        ot_name.to_string()
    }

    fn set_ot_name(&mut self, ot_name: &[u8]) {
        self.ot_name[..ot_name.len()].copy_from_slice(ot_name);
    }

    pub fn checksum(&self) -> u16 {
        LittleEndian::read_u16(&self.checksum)
    }

    pub fn species(&self) -> String {
        let dex_num = self.nat_dex_number();

        if dex_num != 0 {
            match pk_species(dex_num) {
                Ok(species) => species,
                Err(_) => String::from(""),
            }
        } else {
            String::from("")
        }
    }

    pub fn set_species(&mut self, species: &str) -> Result<(), PokemonError> {

        if self.species().to_uppercase() == self.nickname() {
            self.set_nickname(&species.to_uppercase());
        }

        let mut id = match nat_dex_num(species) {
            Ok(id) => id,
            Err(e) => return Err(PokemonError::UnknownSpecies(species.to_string())),
        };

        if id == 0 {
            id = 412;
        } else if id >= 252 {
            id = SPECIES[(id as usize).saturating_sub(251)];
        }

        let offset = self.pokemon_data.growth_offset;
        self.pokemon_data.data[offset..offset + 2].copy_from_slice(&id.to_le_bytes());

        Ok(())
    }

    pub fn nat_dex_number(&self) -> u16 {
        let species = self.species_id();
        if species == 412 {
            return 0;
        }
        if species >= 277 {
            return (SPECIES
                .iter()
                .position(|&x| x == species)
                .unwrap()
                .saturating_add(251))
            .try_into()
            .unwrap();
        }

        species
    }

    pub fn experience(&self) -> u32 {
        let offset = self.pokemon_data.growth_offset;
        LittleEndian::read_u32(&self.pokemon_data.data[offset + 4..offset + 8])
    }

    pub fn gender(&self) -> Gender {
        gender_from_p(self.personality_value(), self.nat_dex_number())
    }

    pub fn level(&self) -> u8 {
        let mut level: u32 = 0;

        if self.is_empty() {
            return level as u8;
        }

        let index = self.nat_dex_number();

        let growth = match growth_rate(index) {
            Ok(growth) => growth,
            Err(_) => String::from(""),
        };

        let growth_index = growth_index(&growth);

        let experience = self.experience();

        let mut iter = EXPERIENCE_TABLE.iter().peekable();

        while let Some(current) = iter.next() {
            if let Some(peek) = iter.peek() {
                if current[growth_index] <= experience && experience < peek[growth_index] {
                    level = current[6];
                    break;
                }
            } else {
                level = current[6];
            }
        }

        level as u8
    }

    pub fn set_level(&mut self, level: u8) {
        let index = self.nat_dex_number();
        println!("nat_dex_number: {:?}", &index);
        let growth = match growth_rate(index) {
            Ok(growth) => growth,
            Err(_) => String::from(""),
        };

        let growth_index = growth_index(&growth);
        println!("level: {:?}, growth_index: {:?}", &level, &growth_index);
        let experience = EXPERIENCE_TABLE[(level - 1) as usize][growth_index];

        let offset = self.pokemon_data.growth_offset;
        self.pokemon_data.data[offset + 4..offset + 8].copy_from_slice(&experience.to_le_bytes());
    }

    pub fn typing(&self) -> Option<(String, Option<String>)> {
        if self.is_empty() {
            return None;
        }

        let index = self.nat_dex_number();

        match typing(index) {
            Ok(typing) => Some(typing),
            Err(_) => None,
        }
    }

    pub fn ability(&self) -> String {
        let index = self.nat_dex_number();
        let ability_index = self.ability_index();

        match ability_index {
            0 => match ability(index) {
                Ok(ability) => ability,
                Err(_) => String::from(""),
            },
            1 => match hidden_ability(index) {
                Ok(ability) => ability,
                Err(_) => String::from(""),
            },
            _ => String::from(""),
        }
    }

    pub fn moves(&self) -> Vec<(String, String, u8, u8)> {
        let offset = self.pokemon_data.attacks_offset;
        let move1_index =
            LittleEndian::read_u16(&self.pokemon_data.data[offset..offset + 2]) as usize;
        let move2_index =
            LittleEndian::read_u16(&self.pokemon_data.data[offset + 2..offset + 4]) as usize;
        let move3_index =
            LittleEndian::read_u16(&self.pokemon_data.data[offset + 4..offset + 6]) as usize;
        let move4_index =
            LittleEndian::read_u16(&self.pokemon_data.data[offset + 6..offset + 8]) as usize;

        let pp1 = &self.pokemon_data.data[offset + 8..offset + 9];
        let pp2 = &self.pokemon_data.data[offset + 9..offset + 10];
        let pp3 = &self.pokemon_data.data[offset + 10..offset + 11];
        let pp4 = &self.pokemon_data.data[offset + 11..offset + 12];

        let mut moves: Vec<(String, String, u8, u8)> = vec![];

        let move1 = match move_data(move1_index) {
            Ok(m) => Some(m),
            Err(_) => None,
        };
        let move2 = match move_data(move2_index) {
            Ok(m) => Some(m),
            Err(_) => None,
        };
        let move3 = match move_data(move3_index) {
            Ok(m) => Some(m),
            Err(_) => None,
        };
        let move4 = match move_data(move4_index) {
            Ok(m) => Some(m),
            Err(_) => None,
        };

        if let Some(p_move) = move1 {
            moves.push((p_move.0, p_move.1, pp1[0], p_move.2));
        }

        if let Some(p_move) = move2 {
            moves.push((p_move.0, p_move.1, pp2[0], p_move.2));
        }

        if let Some(p_move) = move3 {
            moves.push((p_move.0, p_move.1, pp3[0], p_move.2));
        }

        if let Some(p_move) = move4 {
            moves.push((p_move.0, p_move.1, pp4[0], p_move.2));
        }

        moves
    }

    pub fn set_move(&mut self, position: usize, attack: &str) {
        if let Some(p_move) = MOVES.iter().find(|&m| m["ename"] == attack) {
            let offset = self.pokemon_data.attacks_offset;

            let index = p_move["id"].as_u64().unwrap() as u16;
            let pp = p_move["pp"].as_u64().unwrap() as u8;

            self.pokemon_data.data[offset + (position * 2)..offset + ((position * 2) + 2)]
                .copy_from_slice(&index.to_le_bytes());

            self.pokemon_data.data[offset + (position + 8)..offset + (position + 9)]
                .copy_from_slice(&pp.to_le_bytes());
        }
    }

    pub fn held_item(&self) -> String {
        let offset = self.pokemon_data.growth_offset;
        let held_item_index =
            LittleEndian::read_u16(&self.pokemon_data.data[offset + 2..offset + 4]) as usize;

        if held_item_index == 0 {
            return String::from("-");
        }

        match find_item(held_item_index) {
            Ok(i) => i,
            Err(_) => String::from("-"),
        }
    }

    pub fn pokeball_caught(&self) -> usize {
        let offset = self.pokemon_data.miscellaneous_offset;
        let origins_info = &self.pokemon_data.data[offset + 2..offset + 4];
        // mask to get the bits 11 - 14
        //0x7800 = 0b0111100000000000
        const BITS_MASK: u16 = 0x7800;

        ((LittleEndian::read_u16(origins_info) & BITS_MASK) >> 11) as usize
    }

    fn set_pokeball_caught(&mut self, ball_id: u16) {
        let offset = self.pokemon_data.miscellaneous_offset;
        self.pokemon_data.data[offset + 2..offset + 4]
            .copy_from_slice(&(ball_id << 11).to_le_bytes())
    }

    pub fn pokerus_status(&self) -> Pokerus {
        let offset = self.pokemon_data.miscellaneous_offset;
        let pokerus = &self.pokemon_data.data[offset..offset + 1];
        //strain    = 0xF0  = 0b11110000
        //days left = 0xF   = 0b00001111
        const DAYS_MASK: u8 = 0xF;
        const STRAIN_MASK: u8 = 0xF0;

        let strain: u8 = (pokerus[0] & STRAIN_MASK) >> 4;
        let days: u8 = pokerus[0] & DAYS_MASK;

        if strain > 0 && days == 0 {
            Pokerus::Cured
        } else if strain > 0 && days > 0 {
            Pokerus::Infected
        } else {
            Pokerus::None
        }
    }

    pub fn nature(&self) -> String {
        if !self.is_empty() {
            let p = self.nature_index();

            NATURE[p].to_string()
        } else {
            String::from("")
        }
    }

    // doesn't work, don't know why!!
    // generating PIDs is buggy, still don't understand why or how
    pub fn set_nature(&mut self, nature: &str) {
        //let nature_index = NATURE.iter().position(|n| n == &nature).unwrap();
        //let new_p = ((self.personality_value() / 100) * 100) + nature_index as u32;
        let mut seed: u32 = 0x5A0;
        let new_p = loop {
            let personality_value = gen_p(&mut seed);

            let p = (personality_value % 25) as usize;
            let new_nature = NATURE[p].to_string();

            let new_gender = gender_from_p(personality_value, self.nat_dex_number());

            if nature == new_nature && self.gender() == new_gender {
                break personality_value;
            }
        };

        self.personality_value.copy_from_slice(&new_p.to_le_bytes());
    }

    fn save_stats(&mut self) {
        let ev_offset = self.pokemon_data.ev_offset;
        let iv_offset = self.pokemon_data.miscellaneous_offset;

        self.pokemon_data.data[ev_offset..ev_offset + 1].copy_from_slice(&[self.stats.hp_ev as u8]);
        self.pokemon_data.data[ev_offset + 1..ev_offset + 2]
            .copy_from_slice(&[self.stats.attack_ev as u8]);
        self.pokemon_data.data[ev_offset + 2..ev_offset + 3]
            .copy_from_slice(&[self.stats.defense_ev as u8]);
        self.pokemon_data.data[ev_offset + 4..ev_offset + 5]
            .copy_from_slice(&[self.stats.sp_attack_ev as u8]);
        self.pokemon_data.data[ev_offset + 5..ev_offset + 6]
            .copy_from_slice(&[self.stats.sp_defense_ev as u8]);
        self.pokemon_data.data[ev_offset + 3..ev_offset + 4]
            .copy_from_slice(&[self.stats.speed_ev as u8]);

        let mut ivs: u32 = 0;

        ivs |= self.stats.hp_iv as u32;
        ivs |= (self.stats.attack_iv as u32) << 5;
        ivs |= (self.stats.defense_iv as u32) << 10;
        ivs |= (self.stats.speed_iv as u32) << 15;
        ivs |= (self.stats.sp_attack_iv as u32) << 20;
        ivs |= (self.stats.sp_defense_iv as u32) << 25;

        self.pokemon_data.data[iv_offset + 4..iv_offset + 8].copy_from_slice(&ivs.to_le_bytes());
    }

    pub fn stats(&self) -> Stats {
        self.stats
    }

    pub fn stats_mut(&mut self) -> &mut Stats {
        &mut self.stats
    }

    fn init_stats(&mut self) {
        let index = self.nat_dex_number().saturating_sub(1) as usize;
        let nature_index = self.nature_index();

        let base_stats = &POKEDEX_JSON[index]["base"];
        let ev_offset = self.pokemon_data.ev_offset;

        let iv_offset = self.pokemon_data.miscellaneous_offset;

        let ivs = LittleEndian::read_u32(&self.pokemon_data.data[iv_offset + 4..iv_offset + 8]);

        self.stats = Stats {
            // Base
            hp: base_stats["HP"].as_u64().unwrap() as u16,
            attack: base_stats["Attack"].as_u64().unwrap() as u16,
            defense: base_stats["Defense"].as_u64().unwrap() as u16,
            sp_attack: base_stats["Sp. Attack"].as_u64().unwrap() as u16,
            sp_defense: base_stats["Sp. Defense"].as_u64().unwrap() as u16,
            speed: base_stats["Speed"].as_u64().unwrap() as u16,
            // Effort Values
            hp_ev: self.pokemon_data.data[ev_offset..ev_offset + 1][0] as u16,
            attack_ev: self.pokemon_data.data[ev_offset + 1..ev_offset + 2][0] as u16,
            defense_ev: self.pokemon_data.data[ev_offset + 2..ev_offset + 3][0] as u16,
            sp_attack_ev: self.pokemon_data.data[ev_offset + 4..ev_offset + 5][0] as u16,
            sp_defense_ev: self.pokemon_data.data[ev_offset + 5..ev_offset + 6][0] as u16,
            speed_ev: self.pokemon_data.data[ev_offset + 3..ev_offset + 4][0] as u16,
            // Individual Values
            // HP           0x1F        = 0b00000000000000000000000000011111
            // Attack       0x3E0       = 0b00000000000000000000001111100000
            // Defense      0x7C00      = 0b00000000000000000111110000000000
            // Speed        0xF8000     = 0b00000000000011111000000000000000
            // Sp Attack    0x1F00000   = 0b00000001111100000000000000000000
            // Sp Defense   0x3E000000  = 0b00111110000000000000000000000000
            hp_iv: (ivs & 0x1F) as u16,
            attack_iv: ((ivs & 0x3E0) >> 5) as u16,
            defense_iv: ((ivs & 0x7C00) >> 10) as u16,
            speed_iv: ((ivs & 0xF8000) >> 15) as u16,
            sp_attack_iv: ((ivs & 0x1F00000) >> 20) as u16,
            sp_defense_iv: ((ivs & 0x3E000000) >> 25) as u16,
            // Nature Modifiers
            n_mod: NATURE_MODIFIER[nature_index],
        };
    }

    pub fn friendship(&self) -> u8 {
        let offset = self.pokemon_data.growth_offset;
        self.pokemon_data
            .data
            .get(offset + 9..offset + 10)
            .unwrap_or_default()[0]
    }

    pub fn set_friendship(&mut self, value: u8) {
        let offset = self.pokemon_data.growth_offset;
        self.pokemon_data.data[offset + 9..offset + 10].copy_from_slice(&value.to_le_bytes());
    }

    pub fn personality_value(&self) -> u32 {
        LittleEndian::read_u32(&self.personality_value)
    }

    fn set_personality_value(&mut self, value: u32) {
        self.personality_value.copy_from_slice(&value.to_le_bytes());
    }

    pub fn infect_pokerus(&mut self) {
        let offset = self.pokemon_data.miscellaneous_offset;
        let mut pokerus = self.pokemon_data.data[offset..offset + 1][0];

        let mut rng = rand::thread_rng();
        let strain = rng.gen_range(1..=15);
        let days = (strain % 4) + 1;

        pokerus |= strain << 4;
        pokerus |= days;

        self.pokemon_data.data[offset..offset + 1].copy_from_slice(&[pokerus]);
    }

    pub fn cure_pokerus(&mut self) {
        let offset = self.pokemon_data.miscellaneous_offset;
        let mut pokerus = self.pokemon_data.data[offset..offset + 1][0];

        //strain    = 0xF0  = 0b11110000
        const STRAIN_MASK: u8 = 0xF0;

        pokerus &= STRAIN_MASK;

        self.pokemon_data.data[offset..offset + 1].copy_from_slice(&[pokerus]);
    }

    pub fn remove_pokerus(&mut self) {
        let offset = self.pokemon_data.miscellaneous_offset;
        self.pokemon_data.data[offset..offset + 1].copy_from_slice(&[0]);
    }

    pub fn give_item(&mut self, item: &str) {
        let offset = self.pokemon_data.growth_offset;
        let held_item_index = if item == "-" {
            0
        } else {
            item_id_g3(item).unwrap_or(0)
        };

        self.pokemon_data.data[offset + 2..offset + 4]
            .copy_from_slice(&held_item_index.to_le_bytes())
    }

    pub fn raw_data(&self) -> [u8; 80] {
        let mut raw_data: [u8; 80] = [0; 80];
        let mut data: [u8; 48] = [0; 48];

        let ecryption_key =
            LittleEndian::read_u32(&self.personality_value) ^ LittleEndian::read_u32(&self.ot_id);

        pokemon_data_encryption(ecryption_key, &self.pokemon_data.data, &mut data);

        raw_data[0x00..0x04].copy_from_slice(&self.personality_value);
        raw_data[0x04..0x08].copy_from_slice(&self.ot_id);
        raw_data[0x08..0x12].copy_from_slice(&self.nickname);
        raw_data[0x12..0x13].copy_from_slice(&self.language);
        raw_data[0x13..0x14].copy_from_slice(&self.misc_flags);
        raw_data[0x14..0x1B].copy_from_slice(&self.ot_name);
        raw_data[0x1B..0x1C].copy_from_slice(&self.markings);
        raw_data[0x1C..0x1E].copy_from_slice(&self.checksum);
        raw_data[0x1E..0x20].copy_from_slice(&self._padding);
        raw_data[0x20..0x50].copy_from_slice(&data);

        raw_data
    }

    pub fn update_checksum(&mut self) {
        self.save_stats();
        self.checksum
            .copy_from_slice(&self.pokemon_data.checksum().to_le_bytes())
    }

    pub fn lowest_level(&self) -> u8 {
        let mut level: u8 = 1;
        if !self.is_empty() {
            let index = self.nat_dex_number().saturating_sub(1) as usize;

            let prev_evo = &POKEDEX_JSON[index]["evolution"]["prev"];

            if prev_evo != &Value::Null && prev_evo[1].as_str().unwrap().contains("Level") {
                let mut level_str = prev_evo[1].as_str().unwrap().to_string();
                level_str.retain(|c| c.is_numeric());
                level = level_str.parse::<u8>().unwrap();
            }
        }

        level
    }

    pub fn is_egg(&self) -> bool {
        let offset = self.pokemon_data.miscellaneous_offset;
        let iv_egg_ability = &self.pokemon_data.data[offset + 4..offset + 8];
        // mask to get the 30 bit
        // 0x40000000 = 0b01000000000000000000000000000000
        const LOW_1_BITS_MASK: u32 = 0x40000000;

        let bit_value = ((LittleEndian::read_u32(iv_egg_ability) & LOW_1_BITS_MASK) >> 30) as usize;

        if self.is_empty() || bit_value == 0 {
            return false;
        }

        true
    }

    pub fn is_empty(&self) -> bool {
        if self.personality_value.is_empty() || self.personality_value() == 0 {
            return true;
        }

        false
    }

    fn nature_index(&self) -> usize {
        (self.personality_value() % 25) as usize
    }

    fn species_id(&self) -> u16 {
        let offset = self.pokemon_data.growth_offset;
        LittleEndian::read_u16(&self.pokemon_data.data[offset..offset + 2])
    }

    fn ability_index(&self) -> usize {
        let offset = self.pokemon_data.miscellaneous_offset;
        let iv_egg_ability = &self.pokemon_data.data[offset + 4..offset + 8];
        // mask to get the 31 bit
        //0x80000000 = 0b10000000000000000000000000000000
        const LOW_1_BITS_MASK: u32 = 0x80000000;
        // mask out the ability bit and shift it to the right
        ((LittleEndian::read_u32(iv_egg_ability) & LOW_1_BITS_MASK) >> 31) as usize
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PokemonData {
    data: [u8; 48],
    _offset: usize,
    growth_offset: usize,
    attacks_offset: usize,
    ev_offset: usize,
    miscellaneous_offset: usize,
}

impl PokemonData {
    fn new(data: [u8; 48]) -> Self {
        PokemonData {
            data,
            ..PokemonData::default()
        }
    }

    fn set_growth_offset(&mut self, growth_offset: usize) -> &mut Self {
        self.growth_offset = growth_offset;
        self
    }

    fn set_attacks_offset(&mut self, attacks_offset: usize) -> &mut Self {
        self.attacks_offset = attacks_offset;
        self
    }

    fn set_ev_offset(&mut self, ev_offset: usize) -> &mut Self {
        self.ev_offset = ev_offset;
        self
    }

    fn set_miscellaneous_offset(&mut self, miscellaneous_offset: usize) -> &mut Self {
        self.miscellaneous_offset = miscellaneous_offset;
        self
    }

    // calculate checksum
    // To validate the checksum given in the encapsulating Pokémon data structure, the entirety of the four unencrypted data substructures must be summed into a 16-bit value
    // We accomplish this by splitting the unencrypted data substructure in chunks of 2 bytes (u16) and summing every chunk
    fn checksum(&self) -> u16 {
        let mut checksum: u16 = 0;
        for chunk in self.data.chunks(2) {
            let (sum, _) = checksum.overflowing_add(LittleEndian::read_u16(chunk));
            checksum = sum;
        }

        checksum
    }
}

impl Default for PokemonData {
    fn default() -> Self {
        PokemonData {
            data: [0; 48],
            _offset: 0x20,
            growth_offset: 0,
            attacks_offset: 0,
            ev_offset: 0,
            miscellaneous_offset: 0,
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Stats {
    // Base
    hp: u16,
    attack: u16,
    defense: u16,
    sp_attack: u16,
    sp_defense: u16,
    speed: u16,
    // Effort Values
    pub hp_ev: u16,
    pub attack_ev: u16,
    pub defense_ev: u16,
    pub sp_attack_ev: u16,
    pub sp_defense_ev: u16,
    pub speed_ev: u16,
    // Individual Values
    pub hp_iv: u16,
    pub attack_iv: u16,
    pub defense_iv: u16,
    pub sp_attack_iv: u16,
    pub sp_defense_iv: u16,
    pub speed_iv: u16,
    // Nature Modifiers
    n_mod: [f32; 5],
}

impl Stats {
    pub fn hp(&self, level: u8) -> u16 {
        let level: u16 = level as u16;

        (((2 * self.hp + self.hp_iv + (self.hp_ev / 4)) * level) / 100) + level + 10
    }

    pub fn attack(&self, level: u8) -> u16 {
        calc_stat(
            self.attack,
            self.attack_iv,
            self.attack_ev,
            self.n_mod[0],
            level,
        )
    }

    pub fn defense(&self, level: u8) -> u16 {
        calc_stat(
            self.defense,
            self.defense_iv,
            self.defense_ev,
            self.n_mod[1],
            level,
        )
    }

    pub fn speed(&self, level: u8) -> u16 {
        calc_stat(
            self.speed,
            self.speed_iv,
            self.speed_ev,
            self.n_mod[2],
            level,
        )
    }

    pub fn sp_attack(&self, level: u8) -> u16 {
        calc_stat(
            self.sp_attack,
            self.sp_attack_iv,
            self.sp_attack_ev,
            self.n_mod[3],
            level,
        )
    }

    pub fn sp_defense(&self, level: u8) -> u16 {
        calc_stat(
            self.sp_defense,
            self.sp_defense_iv,
            self.sp_defense_ev,
            self.n_mod[4],
            level,
        )
    }

    pub fn highest_stat(&self, level: u8) -> (&'static str, u16) {
        let mut stats = [
            ("HP", self.hp(level)),
            ("Attack", self.attack(level)),
            ("Defense", self.defense(level)),
            ("Sp. Attack", self.sp_attack(level)),
            ("Sp. Defense", self.sp_defense(level)),
            ("Speed", self.speed(level)),
        ];

        stats.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        stats[0]
    }

    pub fn update_ivs(&mut self, iv: &str, new_iv: u16) {
        match iv {
            "HP" => {
                self.hp_iv = recalc_iv(new_iv);
            }
            "Attack" => {
                self.attack_iv = recalc_iv(new_iv);
            }
            "Defense" => {
                self.defense_iv = recalc_iv(new_iv);
            }
            "Sp. Atk" => {
                self.sp_attack_iv = recalc_iv(new_iv);
            }
            "Sp. Def" => {
                self.sp_defense_iv = recalc_iv(new_iv);
            }
            "Speed" => {
                self.speed_iv = recalc_iv(new_iv);
            }
            _ => {}
        }
    }

    pub fn update_evs(&mut self, ev: &str, new_ev: u16) {
        match ev {
            "HP" => {
                let new_total = new_ev
                    + self.attack_ev
                    + self.defense_ev
                    + self.sp_attack_ev
                    + self.sp_defense_ev
                    + self.speed_ev;

                self.hp_ev = recalc_ev(new_ev, new_total);
            }
            "Attack" => {
                let new_total = new_ev
                    + self.hp_ev
                    + self.defense_ev
                    + self.sp_attack_ev
                    + self.sp_defense_ev
                    + self.speed_ev;

                self.attack_ev = recalc_ev(new_ev, new_total);
            }
            "Defense" => {
                let new_total = new_ev
                    + self.hp_ev
                    + self.attack_ev
                    + self.sp_attack_ev
                    + self.sp_defense_ev
                    + self.speed_ev;

                self.defense_ev = recalc_ev(new_ev, new_total);
            }
            "Sp. Atk" => {
                let new_total = new_ev
                    + self.hp_ev
                    + self.attack_ev
                    + self.defense_ev
                    + self.sp_defense_ev
                    + self.speed_ev;

                self.sp_attack_ev = recalc_ev(new_ev, new_total);
            }
            "Sp. Def" => {
                let new_total = new_ev
                    + self.hp_ev
                    + self.attack_ev
                    + self.defense_ev
                    + self.sp_attack_ev
                    + self.speed_ev;

                self.sp_defense_ev = recalc_ev(new_ev, new_total);
            }
            "Speed" => {
                let new_total = new_ev
                    + self.hp_ev
                    + self.attack_ev
                    + self.defense_ev
                    + self.sp_attack_ev
                    + self.sp_defense_ev;

                self.speed_ev = recalc_ev(new_ev, new_total);
            }
            _ => {}
        }
    }
}

#[derive(Debug)]
pub enum Language {
    Japanese,
    English,
    French,
    Italian,
    German,
    Unused,
    Spanish,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Unused => write!(f, ""),
            Language::French => write!(f, "FRE"),
            Language::German => write!(f, "GER"),
            Language::English => write!(f, "ENG"),
            Language::Italian => write!(f, "ITA"),
            Language::Spanish => write!(f, "SPA"),
            Language::Japanese => write!(f, "JAP"),
        }
    }
}

impl From<[u8; 1]> for Language {
    fn from(language: [u8; 1]) -> Self {
        match language {
            [1] => Language::Japanese,
            [2] => Language::English,
            [3] => Language::French,
            [4] => Language::Italian,
            [5] => Language::German,
            [7] => Language::Spanish,
            _ => Language::Unused,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    M,
    F,
    #[default]
    None,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum Pokerus {
    #[default]
    None,
    Infected,
    Cured,
}

fn order_data_substructure(key: u32, pokemon_data: &mut PokemonData) {
    if key == 0 {
        pokemon_data.set_growth_offset(0);
        pokemon_data.set_attacks_offset(12);
        pokemon_data.set_ev_offset(24);
        pokemon_data.set_miscellaneous_offset(36);
    }
    if key == 1 {
        pokemon_data.set_growth_offset(0);
        pokemon_data.set_attacks_offset(12);
        pokemon_data.set_miscellaneous_offset(24);
        pokemon_data.set_ev_offset(36);
    }
    if key == 2 {
        pokemon_data.set_growth_offset(0);
        pokemon_data.set_ev_offset(12);
        pokemon_data.set_attacks_offset(24);
        pokemon_data.set_miscellaneous_offset(36);
    }
    if key == 3 {
        pokemon_data.set_growth_offset(0);
        pokemon_data.set_ev_offset(12);
        pokemon_data.set_miscellaneous_offset(24);
        pokemon_data.set_attacks_offset(36);
    }
    if key == 4 {
        pokemon_data.set_growth_offset(0);
        pokemon_data.set_miscellaneous_offset(12);
        pokemon_data.set_attacks_offset(24);
        pokemon_data.set_ev_offset(36);
    }
    if key == 5 {
        pokemon_data.set_growth_offset(0);
        pokemon_data.set_miscellaneous_offset(12);
        pokemon_data.set_ev_offset(24);
        pokemon_data.set_attacks_offset(36);
    }
    if key == 6 {
        pokemon_data.set_attacks_offset(0);
        pokemon_data.set_growth_offset(12);
        pokemon_data.set_ev_offset(24);
        pokemon_data.set_miscellaneous_offset(36);
    }
    if key == 7 {
        pokemon_data.set_attacks_offset(0);
        pokemon_data.set_growth_offset(12);
        pokemon_data.set_miscellaneous_offset(24);
        pokemon_data.set_ev_offset(36);
    }
    if key == 8 {
        pokemon_data.set_attacks_offset(0);
        pokemon_data.set_ev_offset(12);
        pokemon_data.set_growth_offset(24);
        pokemon_data.set_miscellaneous_offset(36);
    }
    if key == 9 {
        pokemon_data.set_attacks_offset(0);
        pokemon_data.set_ev_offset(12);
        pokemon_data.set_miscellaneous_offset(24);
        pokemon_data.set_growth_offset(36);
    }
    if key == 10 {
        pokemon_data.set_attacks_offset(0);
        pokemon_data.set_miscellaneous_offset(12);
        pokemon_data.set_growth_offset(24);
        pokemon_data.set_ev_offset(36);
    }
    if key == 11 {
        pokemon_data.set_attacks_offset(0);
        pokemon_data.set_miscellaneous_offset(12);
        pokemon_data.set_ev_offset(24);
        pokemon_data.set_growth_offset(36);
    }
    if key == 12 {
        pokemon_data.set_ev_offset(0);
        pokemon_data.set_growth_offset(12);
        pokemon_data.set_attacks_offset(24);
        pokemon_data.set_miscellaneous_offset(36);
    }
    if key == 13 {
        pokemon_data.set_ev_offset(0);
        pokemon_data.set_growth_offset(12);
        pokemon_data.set_miscellaneous_offset(24);
        pokemon_data.set_attacks_offset(36);
    }
    if key == 14 {
        pokemon_data.set_ev_offset(0);
        pokemon_data.set_attacks_offset(12);
        pokemon_data.set_growth_offset(24);
        pokemon_data.set_miscellaneous_offset(36);
    }
    if key == 15 {
        pokemon_data.set_ev_offset(0);
        pokemon_data.set_attacks_offset(12);
        pokemon_data.set_miscellaneous_offset(24);
        pokemon_data.set_growth_offset(36);
    }
    if key == 16 {
        pokemon_data.set_ev_offset(0);
        pokemon_data.set_miscellaneous_offset(12);
        pokemon_data.set_growth_offset(24);
        pokemon_data.set_attacks_offset(36);
    }
    if key == 17 {
        pokemon_data.set_ev_offset(0);
        pokemon_data.set_miscellaneous_offset(12);
        pokemon_data.set_attacks_offset(24);
        pokemon_data.set_growth_offset(36);
    }
    if key == 18 {
        pokemon_data.set_miscellaneous_offset(0);
        pokemon_data.set_growth_offset(12);
        pokemon_data.set_attacks_offset(24);
        pokemon_data.set_ev_offset(36);
    }
    if key == 19 {
        pokemon_data.set_miscellaneous_offset(0);
        pokemon_data.set_growth_offset(12);
        pokemon_data.set_ev_offset(24);
        pokemon_data.set_attacks_offset(36);
    }
    if key == 20 {
        pokemon_data.set_miscellaneous_offset(0);
        pokemon_data.set_attacks_offset(12);
        pokemon_data.set_growth_offset(24);
        pokemon_data.set_ev_offset(36);
    }
    if key == 21 {
        pokemon_data.set_miscellaneous_offset(0);
        pokemon_data.set_attacks_offset(12);
        pokemon_data.set_ev_offset(24);
        pokemon_data.set_growth_offset(36);
    }
    if key == 22 {
        pokemon_data.set_miscellaneous_offset(0);
        pokemon_data.set_ev_offset(12);
        pokemon_data.set_growth_offset(24);
        pokemon_data.set_attacks_offset(36);
    }
    if key == 23 {
        pokemon_data.set_miscellaneous_offset(0);
        pokemon_data.set_ev_offset(12);
        pokemon_data.set_attacks_offset(24);
        pokemon_data.set_growth_offset(36);
    }
}

fn pokemon_data_encryption(key: u32, data: &[u8], new_data: &mut [u8]) {
    // second we decrypt the data by XORing it, 32 bits (or 4 bytes) at a time with the encryption key
    // We accomplish this by splitting the data substructure in chunks of 4 bytes (u32) and XORing every chunk
    for (i, chunk) in data.chunks(4).enumerate() {
        let i_offest = 4 * i;
        let xored = LittleEndian::read_u32(chunk) ^ key;
        new_data[i_offest..i_offest + 4].copy_from_slice(&xored.to_le_bytes());
    }
}

fn growth_index(growth: &str) -> usize {
    match growth {
        "Erratic" => 0,
        "Fast" => 1,
        "Medium Fast" => 2,
        "Medium Slow" => 3,
        "Slow" => 4,
        "Fluctuating" => 5,
        _ => 7,
    }
}

fn gender_threshold(dex_num: u16) -> u32 {
    let gender = match gender_ratio(dex_num) {
        Ok(ratio) => ratio,
        Err(_) => String::from(""),
    };

    let mut iter = GENDER_THRESHOLD
        .iter()
        .filter(|(_, g)| *g.to_string() == gender);

    let Some((threshold, _)) = iter.next() else {
        return 255;
    };

    *threshold
}

fn calc_stat(base: u16, iv: u16, ev: u16, n_mod: f32, level: u8) -> u16 {
    let level: u16 = level as u16;
    (((((2 * base + iv + (ev / 4)) * level) / 100) + 5) as f32 * n_mod).floor() as u16
}

fn recalc_ev(new_ev: u16, new_total: u16) -> u16 {
    if new_total < 510 && new_ev < 252 {
        new_ev
    } else if new_total < 510 && new_ev > 252 {
        new_ev.saturating_sub(new_ev.saturating_sub(252))
    } else {
        new_ev.saturating_sub(new_total.saturating_sub(510))
    }
}

fn recalc_iv(new_iv: u16) -> u16 {
    if new_iv < 31 {
        new_iv
    } else {
        new_iv.saturating_sub(new_iv.saturating_sub(31))
    }
}

// generating PIDs is buggy, still don't understand why or how
pub fn gen_pokemon_from_species(
    pokemon_offset: usize,
    species: &str,
    ot_name: &[u8],
    ot_id: &[u8],
) -> Pokemon {
    let dummy = [
        101, 231, 167, 198, 154, 166, 220, 6, 206, 201, 204, 189, 194, 195, 189, 255, 1, 0, 2, 2,
        195, 213, 226, 255, 255, 255, 255, 0, 49, 30, 0, 0, 255, 65, 123, 193, 255, 65, 123, 192,
        255, 65, 123, 192, 231, 64, 123, 192, 103, 65, 123, 192, 255, 7, 123, 192, 255, 81, 254,
        225, 69, 32, 147, 217, 255, 65, 123, 192, 245, 65, 86, 192, 255, 65, 123, 192, 220, 105,
        123, 192, 0, 0, 0, 0, 5, 255, 20, 0, 20, 0, 11, 0, 10, 0, 9, 0, 14, 0, 10, 0,
    ];
    //587584645
    //428966877
    //565740844
    //2590028500
    //926709307
    let mut seed: u32 = 0x5A0;

    let p: u32 = gen_p(&mut seed);

    let mut new_pokemon = Pokemon::new(pokemon_offset, &dummy);

    new_pokemon.set_personality_value(p);
    new_pokemon.set_personality_value(587584645);

    new_pokemon.set_species(species);
    new_pokemon.set_level(new_pokemon.lowest_level());
    new_pokemon.set_pokeball_caught(4);
    new_pokemon.set_ot_id(ot_id);
    new_pokemon.set_ot_name(ot_name);
    new_pokemon.set_nickname(&species.to_uppercase());

    new_pokemon.init_stats();

    new_pokemon.update_checksum();

    new_pokemon
}

const MULTIPLIER: u32 = 1103515245;
//const INVERSE_MULTIPLIER: u32 = 4005161829;
const INCREMENT: u32 = 24691;

fn rng(state: &mut u32) -> u32 {
    *state = state.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT);
    *state >> 16
}

/*fn anti_rng(state: u32) -> u32 {
    let rng = INVERSE_MULTIPLIER.wrapping_mul(state.wrapping_sub(INCREMENT));
    rng >> 16
}*/

// generating PIDs is buggy, still don't understand why or how
fn gen_p(seed: &mut u32) -> u32 {
    let mut t_rng = rand::thread_rng();
    // for some still unknown reason, the program has a strange behaviour que using some ranbom number to generate a PID
    let mut seed: u32 = t_rng.gen();
    let p_h: u32 = rng(&mut seed);
    let p_l: u32 = rng(&mut seed);

    p_l | (p_h << 16)
}

fn gender_from_p(p: u32, dex_num: u16) -> Gender {
    let pg = p % 256;

    if p == 0 {
        Gender::None
    } else {
        let threshold = gender_threshold(dex_num);

        if threshold == 255 {
            Gender::None
        } else if pg >= threshold {
            Gender::M
        } else {
            Gender::F
        }
    }
}
