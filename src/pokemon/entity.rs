//! The core [`Pokemon`] struct and its full API.
//!
//! [`Pokemon`] is a 100-byte (party) or 80-byte (PC) structure. The first 80 bytes are always
//! present; the last 20 bytes (status condition, level mirror, current/total HP, and combat stats)
//! are only stored for party Pokémon. [`Pokemon::from_bytes`] handles both sizes automatically.
//!
//! Internally, bytes 0x20–0x4F hold four 12-byte sub-structures (see [`crate::pokemon::data`])
//! that are XOR-encrypted with `PID ^ OT_ID` and shuffled according to `PID % 24`.

use crate::error::PokemonError;
use crate::pokemon::data::*;
use crate::pokemon::factory::generate_method_1;
use crate::pokemon::stats::Stats;
use crate::save::trainer::TrainerID;
use byteorder::{ByteOrder, LittleEndian};
use rand::Rng;
use std::fmt;

use crate::pokemon::crypto::{calculate_checksum, crypt_data};

use crate::common::character_set::{get_char, get_code};
use crate::misc::{
    ability, evolution, find_item, find_move, gender_ratio, growth_rate, hidden_ability,
    item_id_g3, move_data, nat_dex_num, pk_species, typing, EXPERIENCE_TABLE, GENDER_THRESHOLD,
    NATURE, SPECIES,
};

/// A parsed Pokémon from a Generation III save file.
///
/// Encapsulates both the persistent 80-byte data (always present in PC and party storage)
/// and the volatile 20-byte party data (only meaningful when the Pokémon is in the party).
/// Use [`Pokemon::from_bytes`] to parse raw bytes and [`Pokemon::to_bytes`] to serialise back.
#[derive(Debug, Default, Copy, Clone)]
pub struct Pokemon {
    /// Byte offset of this Pokémon within the full save file buffer.
    pub offset: usize,
    /// Personality value (PID). Determines gender, nature, ability, shiny status, and block order.
    pub personality_value: u32,
    /// Original trainer ID bytes (little-endian: public u16, then secret u16).
    ot_id: [u8; 4],
    /// Nickname encoded in Gen III character set (10 bytes, space-padded).
    nickname: [u8; 10],
    /// Language code byte (see [`crate::pokemon::data::Language`]).
    language: u8,
    /// Miscellaneous flags byte (bit 0 = bad egg, etc.).
    misc_flags: u8,
    /// OT name encoded in Gen III character set (7 bytes, space-padded).
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
    /// Calculated battle stats (HP, Attack, Defense, etc.) and their IV/EV components.
    pub stats: Stats,
}

impl fmt::Display for Pokemon {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ivs = self.ivs();
        let evs = &self.data.evs;
        let stats = &self.stats;

        // Basic Info
        writeln!(f, "Nickname:      {}", self.nickname())?;
        writeln!(f, "Species:       {} (#{})", self.species(), self.nat_dex_number())?;
        writeln!(f, "Level:         {}", self.level())?;
        writeln!(f, "Experience:    {}", self.experience())?;
        writeln!(f, "Gender:        {:?}", self.gender())?;
        writeln!(f, "Nature:        {}", self.nature())?;
        writeln!(f, "Ability:       {}", self.ability())?;
        writeln!(f, "Held Item:     {}", self.item())?;
        writeln!(f, "Friendship:    {}", self.friendship())?;
        writeln!(f, "Language:      {}", self.language())?;
        writeln!(f, "Markings:      {:#04b}", self.markings)?; // Binary representation of markings

        // Trainer Info
        writeln!(f, "\n--- TRAINER INFO ---")?;
        writeln!(f, "OT Name:       {}", self.ot_name())?;
        let ot_id: TrainerID = self.ot_id.into();
        writeln!(f, "OT ID:         {:05} (Public), {:05} (Secret)", ot_id.public, ot_id.private)?;
        writeln!(f, "PID:           0x{:08X} ({})", self.personality_value, self.personality_value)?;

        // Status & Health
        writeln!(f, "\n--- STATUS & HEALTH ---")?;
        writeln!(f, "Status:        {:#010b} ({})", self.status_condition, self.status_condition)?; // Binary or hex might be more useful
        writeln!(f, "HP:            {}/{}", self.current_hp, self.total_hp)?;
        writeln!(f, "Pokérus:       {:?}", self.pokerus_status())?;

        // Stats Table
        writeln!(f, "\n--- STATS (Base + IVs + EVs) ---")?;
        writeln!(f, "{:<12} | {:<5} | {:<3} | {:<3}", "Stat", "Value", "IV", "EV")?;
        writeln!(f, "{:-<12}-+-{:-<5}-+-{:-<3}-+-{:-<3}", "", "", "", "")?;

        writeln!(f, "{:<12} | {:<5} | {:<3} | {:<3}", "HP", self.total_hp, ivs.hp_iv(), evs.hp)?;
        writeln!(f, "{:<12} | {:<5} | {:<3} | {:<3}", "Attack", stats.attack, ivs.attack_iv(), evs.attack)?;
        writeln!(f, "{:<12} | {:<5} | {:<3} | {:<3}", "Defense", stats.defense, ivs.defense_iv(), evs.defense)?;
        writeln!(f, "{:<12} | {:<5} | {:<3} | {:<3}", "Sp. Atk", stats.sp_attack, ivs.sp_attack_iv(), evs.sp_attack)?;
        writeln!(f, "{:<12} | {:<5} | {:<3} | {:<3}", "Sp. Def", stats.sp_defense, ivs.sp_defense_iv(), evs.sp_defense)?;
        writeln!(f, "{:<12} | {:<5} | {:<3} | {:<3}", "Speed", stats.speed, ivs.speed_iv(), evs.speed)?;

        let total_evs: u16 = [evs.hp, evs.attack, evs.defense, evs.sp_attack, evs.sp_defense, evs.speed].iter().map(|&x| x as u16).sum();
        writeln!(f, "{:<12} | {:<5} | {:<3} | {:<3}/510", "Total", "-", "-", total_evs)?;

        // Moves
        writeln!(f, "\n--- MOVES ---")?;
        for (i, (name, type_, pp, max_pp)) in self.moves().iter().enumerate() {
            writeln!(f, "Move {}: {:<16} ({}) - PP: {}/{}", i + 1, name, type_, pp, max_pp)?;
        }

        // Origins & Misc
        writeln!(f, "\n--- ORIGINS & MISC ---")?;
        writeln!(f, "Met Location:  {}", self.data.misc.met_location)?;
        writeln!(f, "Level Met:     {}", self.data.misc.origins_info.level_met())?;
        writeln!(f, "Game of Origin:{}", self.data.misc.origins_info.game_of_origin())?; // You might want a lookup for game IDs
        writeln!(f, "Pokéball:      {}", self.pokeball_caught())?; // Consider a lookup for ball names
        writeln!(f, "Is Egg:        {}", self.is_egg())?;
        Ok(())
    }
}

impl Pokemon {
    /// Parses a [`Pokemon`] from a byte slice.
    ///
    /// Accepts both 80-byte (PC format) and 100-byte (party format) slices.
    /// The encrypted sub-structure is decrypted and de-shuffled automatically.
    ///
    /// # Errors
    /// Returns [`PokemonError::InvalidDataLength`] if the slice is shorter than 80 bytes.
    pub fn from_bytes(offset: usize, buffer: &[u8]) -> Result<Self, PokemonError> {
        if buffer.len() < 80 {
            return Err(PokemonError::InvalidDataLength(buffer.len()));
        }

        let personality_value = LittleEndian::read_u32(&buffer[0x00..0x04]);
        let ot_id = LittleEndian::read_u32(&buffer[0x04..0x08]);

        // unencrypt the pokemon data substructure
        // first we obtain the ecryption key by XORing the entire original trainer id with the pokemon personality value
        let key = personality_value ^ ot_id;
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

        let mut pokemon = Pokemon {
            offset,
            personality_value,
            ot_id: buffer[0x04..0x08].try_into().unwrap(),
            nickname: buffer[0x08..0x12].try_into().unwrap(),
            language: buffer[0x12],
            misc_flags: buffer[0x13],
            ot_name: buffer[0x14..0x1B].try_into().unwrap(),
            markings: buffer[0x1B],
            checksum: LittleEndian::read_u16(&buffer[0x1C..0x1E]),
            padding: LittleEndian::read_u16(&buffer[0x1E..0x20]),
            data: PokemonData {
                growth,
                attacks,
                evs,
                misc,
            },
            // Defaults for party data, overridden below if buffer size allows
            ..Default::default()
        };

        pokemon.init_stats();

        if buffer.len() >= 100 {
            pokemon.status_condition = LittleEndian::read_u32(&buffer[0x50..0x54]);
            pokemon.level = buffer[0x54];
            pokemon.mail_id = buffer[0x55];
            pokemon.current_hp = LittleEndian::read_u16(&buffer[0x56..0x58]);
            pokemon.total_hp = LittleEndian::read_u16(&buffer[0x58..0x5A]);
            // Note: We trust our recalc logic for Atk/Def/etc over the file's volatile data
            // but we respect HP/Status as they represent current battle state.
        } else {
            // If loading from PC (80 bytes), ensure full health
            pokemon.current_hp = pokemon.stats.hp(pokemon.level);
            pokemon.total_hp = pokemon.stats.hp(pokemon.level);
            pokemon.status_condition = 0; // Healthy
        }

        Ok(pokemon)
    }

    /// Serialises this [`Pokemon`] into a 100-byte buffer (party format).
    ///
    /// Re-shuffles and re-encrypts the sub-structures using the stored `PID ^ OT_ID` key,
    /// recalculates the checksum, and writes all party data fields. When saving to PC storage,
    /// only the first 80 bytes should be written.
    pub fn to_bytes(&self) -> [u8; 100] {
        let mut buffer = [0u8; 100];

        // 1. Header
        LittleEndian::write_u32(&mut buffer[0x00..0x04], self.personality_value);
        buffer[0x04..0x08].copy_from_slice(&self.ot_id);
        buffer[0x08..0x12].copy_from_slice(&self.nickname);
        buffer[0x12] = self.language;
        buffer[0x13] = self.misc_flags;
        buffer[0x14..0x1B].copy_from_slice(&self.ot_name);
        buffer[0x1B] = self.markings;
        // Checksum written later
        LittleEndian::write_u16(&mut buffer[0x1E..0x20], self.padding);

        // 2. Serialize Substructures
        let mut flat_data = [0u8; 48];
        let g = &self.data.growth;
        let a = &self.data.attacks;
        let e = &self.data.evs;
        let m = &self.data.misc;

        // Growth
        LittleEndian::write_u16(&mut flat_data[0..2], g.species);
        LittleEndian::write_u16(&mut flat_data[2..4], g.item);
        LittleEndian::write_u32(&mut flat_data[4..8], g.experience);
        flat_data[8] = g.pp_bonuses;
        flat_data[9] = g.friendship;
        LittleEndian::write_u16(&mut flat_data[10..12], g.unknown);

        // Attacks
        for (i, move_id) in a.moves.iter().enumerate() {
            LittleEndian::write_u16(&mut flat_data[12 + (i * 2)..14 + (i * 2)], *move_id);
        }
        flat_data[20..24].copy_from_slice(&a.pp);

        // EVs
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

        // Misc
        flat_data[36] = m.pokerus;
        flat_data[37] = m.met_location;
        flat_data[38..40].copy_from_slice(&m.origins_info.into_bytes());
        flat_data[40..44].copy_from_slice(&m.iv_egg_ability.into_bytes());
        flat_data[44..48].copy_from_slice(&m.ribbons_obedience.into_bytes());

        // 3. Calculate Checksum (Atomic)
        let checksum = calculate_checksum(&flat_data);
        LittleEndian::write_u16(&mut buffer[0x1C..0x1E], checksum);

        // 4. Shuffle & Encrypt
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
            let dest_start = i * 12;
            shuffled_data[dest_start..dest_start + 12].copy_from_slice(&flat_data[src_range]);
        }

        let ot_id = LittleEndian::read_u32(&self.ot_id);

        // unencrypt the pokemon data substructure
        // first we obtain the ecryption key by XORing the entire original trainer id with the pokemon personality value
        let key = self.personality_value ^ ot_id;
        crypt_data(&mut shuffled_data, key);
        buffer[0x20..0x50].copy_from_slice(&shuffled_data);

        // 5. Party Data (Volatile)
        // Always write current state. If saving to PC, the caller (SaveData) will truncate this.
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

    /// Recalculates and stores the sub-structure checksum without serialising to bytes.
    ///
    /// Call this after modifying any field in [`Pokemon::data`] to keep the checksum consistent.
    pub fn update_checksum(&mut self) {
        // Serialize Substructures
        let mut flat_data = [0u8; 48];
        let g = &self.data.growth;
        let a = &self.data.attacks;
        let e = &self.data.evs;
        let m = &self.data.misc;

        // Growth
        LittleEndian::write_u16(&mut flat_data[0..2], g.species);
        LittleEndian::write_u16(&mut flat_data[2..4], g.item);
        LittleEndian::write_u32(&mut flat_data[4..8], g.experience);
        flat_data[8] = g.pp_bonuses;
        flat_data[9] = g.friendship;
        LittleEndian::write_u16(&mut flat_data[10..12], g.unknown);

        // Attacks
        for (i, move_id) in a.moves.iter().enumerate() {
            LittleEndian::write_u16(&mut flat_data[12 + (i * 2)..14 + (i * 2)], *move_id);
        }
        flat_data[20..24].copy_from_slice(&a.pp);

        // EVs
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

        // Misc
        flat_data[36] = m.pokerus;
        flat_data[37] = m.met_location;
        flat_data[38..40].copy_from_slice(&m.origins_info.into_bytes());
        flat_data[40..44].copy_from_slice(&m.iv_egg_ability.into_bytes());
        flat_data[44..48].copy_from_slice(&m.ribbons_obedience.into_bytes());

        // 3. Calculate Checksum (Atomic)
        self.checksum = calculate_checksum(&flat_data);
    }
}

impl Pokemon {
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

    pub fn set_nickname(&mut self, nickname: &str) {
        let name: Vec<u8> = format!("{: <10}", nickname)
            .chars()
            .map(|s| get_code(&s.to_string()))
            .collect();
        self.nickname.copy_from_slice(&name);
    }

    pub fn species_id(&self) -> u16 {
        self.data.growth.species
    }

    pub fn species(&self) -> String {
        let dex_num = self.nat_dex_number();
        if dex_num != 0 {
            pk_species(dex_num).unwrap_or_default()
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
            Err(_) => return Err(PokemonError::UnknownSpecies(species.to_string())),
        };

        if id == 0 {
            id = 412;
        } else if id >= 252 {
            id = SPECIES[(id as usize).saturating_sub(251)];
        }

        self.data.growth.species = id;

        if self.lowest_level() > self.level() {
            self.set_level(self.lowest_level());
        }

        Ok(())
    }

    pub fn item(&self) -> String {
        let held_item_index = self.data.growth.item as usize;

        if held_item_index == 0 {
            return String::from("-");
        }

        match find_item(held_item_index) {
            Ok(i) => i,
            Err(_) => String::from("-"),
        }
    }

    pub fn set_item(&mut self, item: &str) -> Result<(), PokemonError> {
        if item == "-" || item == "None" || item.is_empty() {
            self.data.growth.item = 0;
            return Ok(());
        }

        match item_id_g3(item) {
            Ok(id) => {
                self.data.growth.item = id;
                Ok(())
            }
            Err(_) => Err(PokemonError::UnknownItem(item.to_string())),
        }
    }

    pub fn moves(&self) -> Vec<(String, String, u8, u8)> {
        let mut attacks = vec![];
        for (i, current_pp) in self.data.attacks.moves.iter().zip(self.data.attacks.pp) {
            if let Ok(attack) = move_data(*i as usize) {
                attacks.push((attack.0, attack.1, current_pp, attack.2));
            }
        }
        attacks
    }

    pub fn set_move(&mut self, slot: usize, attack: &str) -> Result<(), PokemonError> {
        if slot > 3 {
            return Err(PokemonError::InvalidMoveSlot(slot));
        }

        match find_move(attack) {
            Ok((id, pp)) => {
                self.data.attacks.moves[slot] = id;
                self.data.attacks.pp[slot] = pp;
                Ok(())
            }
            Err(_) => Err(PokemonError::UnknownMove(attack.to_string())),
        }
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

    pub(crate) fn set_ot_name(&mut self, ot_name: &[u8]) {
        let len = ot_name.len().min(7);
        self.ot_name[..len].copy_from_slice(&ot_name[..len]);
    }

    pub fn level(&self) -> u8 {
        self.calculate_level_from_exp()
    }

    pub fn set_level(&mut self, level: u8) {
        let level = level.clamp(1, 100);
        let index = self.nat_dex_number();

        let growth = growth_rate(index).unwrap_or_default();
        let growth_index = growth_index(&growth);

        let experience = EXPERIENCE_TABLE[(level - 1) as usize][growth_index];
        self.data.growth.experience = experience;

        self.init_stats();
    }

    pub fn experience(&self) -> u32 {
        self.data.growth.experience
    }

    pub fn ot_id(&self) -> TrainerID {
        self.ot_id.into()
    }

    pub(crate) fn set_ot_id(&mut self, ot_id: &[u8]) {
        self.ot_id.copy_from_slice(ot_id);
    }

    pub fn ivs(&self) -> IVsEggAbility {
        self.data.misc.iv_egg_ability
    }

    pub fn is_bad_egg(&self) -> bool {
        let flag = self.misc_flags & 0b00000001;

        flag == 1
    }

    pub fn language(&self) -> Language {
        self.language.into()
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

    pub fn gender(&self) -> Gender {
        gender_from_p(self.personality_value, self.nat_dex_number())
    }

    pub fn typing(&self) -> Option<(String, Option<String>)> {
        if self.is_empty() {
            return None;
        }

        let index = self.nat_dex_number();

        typing(index).ok()
    }

    pub fn ability(&self) -> String {
        let index = self.nat_dex_number();
        let ability_index = self.ability_index();

        match ability_index {
            0 => ability(index).unwrap_or_default(),
            1 => hidden_ability(index).unwrap_or_default(),
            _ => String::from(""),
        }
    }

    pub fn pokeball_caught(&self) -> usize {
        // mask to get the bits 11 - 14
        //0x7800 = 0b0111100000000000
        //const BITS_MASK: u16 = 0x7800;
        //((LittleEndian::read_u16(origins_info) & BITS_MASK) >> 11) as usize

        self.data.misc.origins_info.pokeball() as usize
    }

    pub fn set_pokeball_caught(&mut self, ball_id: u8) -> Result<(), PokemonError> {
        if ball_id > 12 {
            return Err(PokemonError::InvalidPokeball(ball_id));
        }

        self.data.misc.origins_info.set_pokeball(ball_id);

        Ok(())
    }

    pub fn pokerus_status(&self) -> Pokerus {
        let pokerus = &self.data.misc.pokerus;
        //strain    = 0xF0  = 0b11110000
        //days left = 0xF   = 0b00001111
        const DAYS_MASK: u8 = 0xF;
        const STRAIN_MASK: u8 = 0xF0;

        let strain: u8 = (pokerus & STRAIN_MASK) >> 4;
        let days: u8 = pokerus & DAYS_MASK;

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

    pub fn set_nature(&mut self, nature: &str) {
        loop {
            // Reroll everything
            let (pid, ivs) = generate_method_1(None);

            // Check if this PID yields the desired Nature
            let p = (pid % 25) as usize;
            let new_nature = NATURE[p];

            // Check if Gender matches (Gender is tied to PID)
            let new_gender = gender_from_p(pid, self.nat_dex_number());
            let new_ability = ivs.ability_bit() as usize;

            if nature == new_nature
                && self.gender() == new_gender
                && self.ability_index() == new_ability
            {
                self.personality_value = pid;
                self.data.misc.iv_egg_ability = ivs;
                self.init_stats(); // Recalculate stats with new Nature/IVs
                self.update_checksum();
                break;
            }
        }
    }

    pub fn stats_mut(&mut self) -> &mut Stats {
        &mut self.stats
    }

    pub fn friendship(&self) -> u8 {
        self.data.growth.friendship
    }

    pub fn set_friendship(&mut self, value: u8) {
        self.data.growth.friendship = value;
    }

    pub fn infect_pokerus(&mut self) {
        let mut pokerus = self.data.misc.pokerus;

        let mut rng = rand::thread_rng();
        let strain = rng.gen_range(1..=15);
        let days = (strain % 4) + 1;

        pokerus |= strain << 4;
        pokerus |= days;

        self.data.misc.pokerus = pokerus;
    }

    pub fn cure_pokerus(&mut self) {
        let mut pokerus = self.data.misc.pokerus;

        //strain    = 0xF0  = 0b11110000
        const STRAIN_MASK: u8 = 0xF0;

        pokerus &= STRAIN_MASK;

        self.data.misc.pokerus = pokerus;
    }

    pub fn remove_pokerus(&mut self) {
        self.data.misc.pokerus = 0;
    }

    pub fn lowest_level(&self) -> u8 {
        let mut level: u8 = 1;
        if !self.is_empty() {
            if let Ok(mut evolution) = evolution(&self.nat_dex_number()) {
                if let Some(prev_level) = evolution.prev_level() {
                    level = prev_level;
                }
            }
        }

        level
    }

    pub fn is_egg(&self) -> bool {
        // mask to get the 30 bit
        // 0x40000000 = 0b01000000000000000000000000000000
        //const LOW_1_BITS_MASK: u32 = 0x40000000;
        //let bit_value = ((LittleEndian::read_u32(iv_egg_ability) & LOW_1_BITS_MASK) >> 30) as usize;

        self.data.misc.iv_egg_ability.is_egg() != 0
    }

    pub fn is_empty(&self) -> bool {
        self.personality_value == 0
    }

    fn nature_index(&self) -> usize {
        (self.personality_value % 25) as usize
    }

    fn ability_index(&self) -> usize {
        // mask to get the 31 bit
        //0x80000000 = 0b10000000000000000000000000000000
        //const LOW_1_BITS_MASK: u32 = 0x80000000;
        // mask out the ability bit and shift it to the right
        //((LittleEndian::read_u32(iv_egg_ability) & LOW_1_BITS_MASK) >> 31) as usize
        self.data.misc.iv_egg_ability.ability_bit() as usize
    }

    fn calculate_level_from_exp(&self) -> u8 {
        let mut level: u32 = 0;

        if self.is_empty() {
            return level as u8;
        }

        let index = self.nat_dex_number();

        let growth = growth_rate(index).unwrap_or_default();

        level = find_level(self.experience(), growth_index(&growth));

        level as u8
    }
}

fn growth_slice_to_u16_array(slice: &[u8], out: &mut [u16; 4]) {
    for i in 0..4 {
        out[i] = LittleEndian::read_u16(&slice[i * 2..(i * 2) + 2]);
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
        _ => 0,
    }
}

fn find_level(target: u32, index: usize) -> u32 {
    let mut i = 0;
    let mut j = 99;
    let mut res = 1;
    while i <= j {
        let mid = (i + j) / 2;
        let value = EXPERIENCE_TABLE[mid][index];

        if value <= target {
            res = EXPERIENCE_TABLE[mid][6];
            i = mid + 1; // Look for a larger `i`
        } else {
            j = mid - 1; // Look for a smaller `i`
        }
    }
    res
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

fn gender_threshold(dex_num: u16) -> u32 {
    let gender = gender_ratio(dex_num).unwrap_or_default();

    let mut iter = GENDER_THRESHOLD
        .iter()
        .filter(|(_, g)| *g.to_string() == gender);

    let Some((threshold, _)) = iter.next() else {
        return 255;
    };

    *threshold
}
