use byteorder::{ByteOrder, LittleEndian};
use csv::Reader;
use lazy_static::lazy_static;
use std::convert::From;
use std::default::Default;
use thiserror::Error;
use std::fmt;

use crate::character_set::get_char;
use crate::misc::PROJECT_DIR;

//const SIGNATURE_MAGIC_NUMBER: usize = 0x08012025;
const NUMBER_GAME_SAVE_SECTIONS: usize = 14;
const SECTION_SIZE: usize = 0x1000; // 4096 bytes
const SECTION_DATA_SIZE: usize = 0x0FF4;
const PC_BUFFER_SECTION_SIZE: usize = 0xF80; // 3968 bytes
const PC_BUFFER_I_SECTION_SIZE: usize = 0x7D0; // 2000 bytes

const GAME_SAVE_A_OFFSET: usize = 0x000000;
//const GAME_SAVE_A_SIZE: usize = 57344;

const GAME_SAVE_B_OFFSET: usize = 0x00E000;
//const GAME_SAVE_B_SIZE: usize = 57344;

//const HALL_FAME_OFFSET: usize = 0x01C000;
//const HALL_FAME_SIZE: usize = 8192;

const SPECIES: [u16; 136] = [
    412, 277, 278, 279, 280, 281, 282, 283, 284, 285, 286, 287, 288, 289, 290, 291, 292, 293, 294,
    295, 296, 297, 298, 299, 300, 304, 305, 309, 310, 392, 393, 394, 311, 312, 306, 307, 364, 365,
    366, 301, 302, 303, 370, 371, 372, 335, 336, 350, 320, 315, 316, 322, 355, 382, 383, 384, 356,
    357, 337, 338, 353, 354, 386, 387, 363, 367, 368, 330, 331, 313, 314, 339, 340, 321, 351, 352,
    308, 332, 333, 334, 344, 345, 358, 359, 380, 379, 348, 349, 323, 324, 326, 327, 318, 319, 388,
    389, 390, 391, 328, 329, 385, 317, 377, 378, 361, 362, 369, 411, 376, 360, 346, 347, 341, 342,
    343, 373, 374, 375, 381, 325, 395, 396, 397, 398, 399, 400, 401, 402, 403, 407, 408, 404, 405,
    406, 409, 410,
];

const GENDER_THRESHOLD: [(u32, &str); 7] = [
    (254, "0:100"),
    (225, "12.5:87.5"),
    (191, "75:25"),
    (127, "50:50"),
    (63, "25:75"),
    (31, "87.5:12.5"),
    (0, "100:0"),
];

//  Erratic[0]  Fast[1] M Fast[2]   M Slow[3]   Slow[4] Fluctuating[5]  Level[6]
pub const EXPERIENCE_TABLE: [[u32; 7]; 7] = [
    [0, 0, 0, 0, 0, 0, 1],
    [15, 6, 8, 9, 10, 4, 2],
    [52, 21, 27, 57, 33, 13, 3],
    [122, 51, 64, 96, 80, 32, 4],
    [237, 100, 125, 135, 156, 65, 5],
    [406, 172, 216, 179, 270, 112, 6],
    [637, 274, 343, 236, 428, 178, 7],
];

lazy_static! {
    pub static ref POKEDEX: Vec<csv::StringRecord> =
        Reader::from_reader(PROJECT_DIR.get_file("pokedex.csv").unwrap().contents())
            .records()
            .map(|record| record.unwrap())
            .collect();
}

pub fn growth_index(growth: &str) -> usize {
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

fn get_gender_threshold(index: usize) -> u32 {
    let gender_m = &POKEDEX[index].get(12);
    let gender_f = &POKEDEX[index].get(13);

    let gender = format!(
        "{}:{}",
        gender_m.unwrap().replace("% male", ""),
        gender_f.unwrap().replace("% female", "").trim()
    );

    let mut iter = GENDER_THRESHOLD
        .iter()
        .filter(|(_, g)| g.to_string() == gender);

    let Some((threshold, _)) = iter.next() else {
        todo!()
    };

    *threshold
}

#[derive(Debug, Clone, Copy)]
pub enum Gender {
    M,
    F,
    None,
}

#[derive(Error, Debug, Clone)]
pub enum DataStructureError {
    #[error("Invalid Index: the index must be between 1..12")]
    InvalidIndex,
}

#[derive(Debug, Copy, Clone)]
pub struct TrainerID {
    public: u16,
    private: u16,
}

impl From<TrainerID> for [u8; 4] {
    fn from(object: TrainerID) -> Self {
        let mut buffer: [u8; 4] = [0; 4];
        // The lower 16 bits represent the visible, public ID.
        // Since it's little endian the lower 16 bit are the first 2 bytes
        // public
        buffer[..2].copy_from_slice(&object.public.to_le_bytes());
        // private
        buffer[2..].copy_from_slice(&object.private.to_le_bytes());

        buffer
    }
}

impl From<[u8; 4]> for TrainerID {
    fn from(buffer: [u8; 4]) -> Self {
        // The lower 16 bits represent the visible, public ID.
        // Since it's little endian the lower 16 bit are the first 2 bytes
        TrainerID {
            public: LittleEndian::read_u16(&buffer[..2]),
            private: LittleEndian::read_u16(&buffer[2..]),
        }
    }
}

/// Specifies the save data being represented
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub enum SectionID {
    #[default]
    TrainerInfo,
    TeamItems,
    GamseState,
    MiscData,
    RivalInfo,
    PCbufferA,
    PCbufferB,
    PCbufferC,
    PCbufferD,
    PCbufferE,
    PCbufferF,
    PCbufferG,
    PCbufferH,
    PCbufferI,
    NA,
}

impl From<u16> for SectionID {
    fn from(id: u16) -> Self {
        match id {
            0 => SectionID::TrainerInfo,
            1 => SectionID::TeamItems,
            2 => SectionID::GamseState,
            3 => SectionID::MiscData,
            4 => SectionID::RivalInfo,
            5 => SectionID::PCbufferA,
            6 => SectionID::PCbufferB,
            7 => SectionID::PCbufferC,
            8 => SectionID::PCbufferD,
            9 => SectionID::PCbufferE,
            10 => SectionID::PCbufferF,
            11 => SectionID::PCbufferG,
            12 => SectionID::PCbufferH,
            13 => SectionID::PCbufferI,
            _ => SectionID::NA,
        }
    }
}

impl From<SectionID> for u16 {
    fn from(id: SectionID) -> Self {
        match id {
            SectionID::TrainerInfo => 0,
            SectionID::TeamItems => 1,
            SectionID::GamseState => 2,
            SectionID::MiscData => 3,
            SectionID::RivalInfo => 4,
            SectionID::PCbufferA => 5,
            SectionID::PCbufferB => 6,
            SectionID::PCbufferC => 7,
            SectionID::PCbufferD => 8,
            SectionID::PCbufferE => 9,
            SectionID::PCbufferF => 10,
            SectionID::PCbufferG => 11,
            SectionID::PCbufferH => 12,
            SectionID::PCbufferI => 13,
            SectionID::NA => 14,
        }
    }
}

impl From<SectionID> for i32 {
    fn from(id: SectionID) -> Self {
        match id {
            SectionID::TrainerInfo => 0,
            SectionID::TeamItems => 1,
            SectionID::GamseState => 2,
            SectionID::MiscData => 3,
            SectionID::RivalInfo => 4,
            SectionID::PCbufferA => 5,
            SectionID::PCbufferB => 6,
            SectionID::PCbufferC => 7,
            SectionID::PCbufferD => 8,
            SectionID::PCbufferE => 9,
            SectionID::PCbufferF => 10,
            SectionID::PCbufferG => 11,
            SectionID::PCbufferH => 12,
            SectionID::PCbufferI => 13,
            SectionID::NA => 14,
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

impl From<Language> for u8 {
    fn from(language: Language) -> Self {
        match language {
            Language::Japanese => 1,
            Language::English => 2,
            Language::French => 3,
            Language::Italian => 4,
            Language::German => 5,
            Language::Unused => 6,
            Language::Spanish => 7,
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Japanese => write!(f, "JAP"),
            Language::English => write!(f, "ENG"),
            Language::French => write!(f, "FRE"),
            Language::Italian => write!(f, "ITA"),
            Language::German => write!(f, "GER"),
            Language::Unused => write!(f, ""),
            Language::Spanish => write!(f, "SPA"),
        }
    }
}

/// All sections contain the same general format: 3968 bytes of data, followed by a footer.
/// -------------------------------
/// | Offset | Size | Contents    |
/// | 0x0000 | 3968 |  Data       |
/// | 0x0FF4 |   2  |  Section ID |
/// | 0x0FF6 |   2  |  Checksum   |
/// | 0x0FF8 |   4  |  Signature  |
/// | 0x0FFC |   4  |  Save Index |
/// -------------------------------
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Default)]
struct Section {
    offset: usize,
    size: usize,
}

impl Section {
    fn get_offset(&self) -> usize {
        self.offset
    }
    fn read_data<'a>(&'a self, buffer: &'a [u8]) -> &'a [u8] {
        let section_buffer = &buffer[self.offset..self.offset + self.size];
        &section_buffer[0..SECTION_DATA_SIZE]
    }

    fn read_data_mut<'a>(&'a self, buffer: &'a mut [u8]) -> &'a mut [u8] {
        let section_buffer = &mut buffer[self.offset..self.offset + self.size];
        &mut section_buffer[0..SECTION_DATA_SIZE]
    }

    fn read_id(&self, buffer: &[u8]) -> SectionID {
        let section_buffer = &buffer[self.offset..self.offset + self.size];
        let id = LittleEndian::read_u16(&section_buffer[0x0FF4..0x0FF6]);
        id.into()
    }

    fn read_save_index(&self, buffer: &[u8]) -> u32 {
        let section_buffer = &buffer[self.offset..self.offset + self.size];
        LittleEndian::read_u32(&section_buffer[0x0FFC..])
    }

    fn _read_checksum(&self, buffer: &[u8]) -> u16 {
        let section_buffer = &buffer[self.offset..self.offset + self.size];
        LittleEndian::read_u16(&section_buffer[0x0FF6..0x0FF8])
    }

    /// Used to validate the integrity of saved data.
    /// A 16-bit checksum generated by adding up bytes from the section. The algorithm is as follows:
    /// -Initialize a 32-bit checksum variable to zero.
    /// -Read 4 bytes at a time as 32-bit word (little-endian) and add it to the variable.
    /// -Take the upper 16 bits of the result, and add them to the lower 16 bits of the result.
    /// -This new 16-bit value is the checksum.
    fn write_checksum(&self, buffer: &mut [u8]) {
        let mut checksum: u32 = 0;
        let data = self.read_data(buffer);

        for chunk in data.chunks(4) {
            let (sum, _) = checksum.overflowing_add(LittleEndian::read_u32(chunk));
            checksum = sum;
        }

        // sum opper and lower bits
        let (checksum, _) = ((checksum & 0xFFFF) as u16).overflowing_add((checksum >> 16) as u16);

        let section_buffer = &mut buffer[self.offset..self.offset + self.size];

        section_buffer[0x0FF6..0x0FF8].copy_from_slice(&checksum.to_le_bytes());
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PokemonData {
    data: [u8; 48],
    offset: usize,
    growth_offset: usize,
    attacks_offset: usize,
    ev_offset: usize,
    miscellaneous_offset: usize,
}

impl PokemonData {
    fn new(data: [u8; 48]) -> Self {
        PokemonData {
            offset: 0x20,
            data,
            growth_offset: 0,
            attacks_offset: 0,
            ev_offset: 0,
            miscellaneous_offset: 0,
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
            offset: 0,
            growth_offset: 0,
            attacks_offset: 0,
            ev_offset: 0,
            miscellaneous_offset: 0,
        }
    }
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
        let xored = LittleEndian::read_u32(&chunk) ^ key;
        new_data[i_offest..i_offest + 4].copy_from_slice(&xored.to_le_bytes());
    }
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
    padding: [u8; 2],
    pokemon_data: PokemonData,
}

impl Pokemon {
    fn new(offset: usize, buffer: &[u8]) -> Self {
        let pokemon_data = &buffer[0x20..0x50];

        // unencrypt the pokemon data substructure
        // first we obtain the ecryption key by XORing the entire original trainer id with the pokemon personality value
        let ecryption_key = LittleEndian::read_u32(&buffer[0x04..0x08])
            ^ LittleEndian::read_u32(&buffer[0x00..0x04]);

        let mut data: [u8; 48] = [0; 48];
        pokemon_data_encryption(ecryption_key, &pokemon_data, &mut data);

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
        let mut padding = [0; 2];

        personality_value.copy_from_slice(&buffer[0x00..0x04]);
        ot_id.copy_from_slice(&buffer[0x04..0x08]);
        nickname.copy_from_slice(&buffer[0x08..0x12]);
        language.copy_from_slice(&buffer[0x12..0x13]);
        misc_flags.copy_from_slice(&buffer[0x13..0x14]);
        ot_name.copy_from_slice(&buffer[0x14..0x1B]);
        markings.copy_from_slice(&buffer[0x1B..0x1C]);
        checksum.copy_from_slice(&buffer[0x1C..0x1E]);
        padding.copy_from_slice(&buffer[0x1E..0x20]);

        Pokemon {
            offset,
            personality_value,
            ot_id,
            nickname,
            language,
            misc_flags,
            ot_name,
            markings,
            checksum,
            padding,
            pokemon_data,
        }
    }

    pub fn read_personality_value(&self) -> u32 {
        LittleEndian::read_u32(&self.personality_value)
    }

    pub fn read_ot_id(&self) -> TrainerID {
        self.ot_id.into()
    }

    pub fn read_name(&self) -> String {
        let nickname = &self
            .nickname
            .iter()
            .map(|c| get_char(*c as usize))
            .collect::<Vec<&str>>();

        let nickname = nickname.join("");
        let nickname = nickname.split(' ').next().unwrap();

        nickname.to_string()
    }

    pub fn read_language(&self) -> Language {
        self.language.into()
    }

    pub fn read_ot_name(&self) -> String {
        let ot_name = &self
            .ot_name
            .iter()
            .map(|c| get_char(*c as usize))
            .collect::<Vec<&str>>();

        let ot_name = ot_name.join("");
        let ot_name = ot_name.split(' ').next().unwrap();

        ot_name.to_string()
    }

    pub fn read_checksum(&self) -> u16 {
        LittleEndian::read_u16(&self.checksum)
    }

    pub fn read_species(&self) -> u16 {
        let offset = self.pokemon_data.growth_offset;
        LittleEndian::read_u16(&self.pokemon_data.data[offset..offset + 2])
    }

    pub fn read_nat_dex_number(&self) -> u16 {
        let species = self.read_species();

        if species == 412 {
            return 0;
        }
        if species >= 277 {
            return (SPECIES.iter().position(|&x| x == species).unwrap() + 251)
                .try_into()
                .unwrap();
        }

        species
    }

    pub fn read_experience(&self) -> u32 {
        let offset = self.pokemon_data.growth_offset;
        LittleEndian::read_u32(&self.pokemon_data.data[offset + 4..offset + 8])
    }

    pub fn is_empty(&self) -> bool {
        if self.personality_value.is_empty() {
            return true;
        } else if self.read_personality_value() == 0 {
            return true;
        }

        false
    }

    pub fn gender(&self) -> Gender {
        let p = self.read_personality_value();
        let pg = p % 256;

        if pg == 255 || p == 0 {
            return Gender::None;
        } else {
            let dex_num = self.read_nat_dex_number();

            let threshold = get_gender_threshold(dex_num.into());
            if pg >= threshold {
                return Gender::M;
            } else {
                return Gender::F;
            }
        }
    }

    pub fn read_level(&self) -> u8 {
        let index = self.read_nat_dex_number().saturating_sub(1);

        let growth = &POKEDEX[index as usize].get(11);

        let growth_index = growth_index(growth.unwrap());

        let experience = self.read_experience();

        let mut iter = EXPERIENCE_TABLE.iter().peekable();
        let mut level = 0;

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

    pub fn species_name(&self) -> String {
        let dex_num = self.read_nat_dex_number();
        let index = dex_num.saturating_sub(1);

        if dex_num != 0{
            POKEDEX[index as usize].get(1).unwrap().to_string()
        } else {
            "".to_string()
        }
    }
}

#[derive(Default)]
pub struct PCBuffer {
    buffer: [Section; 9],
    data: Vec<u8>,
}

impl PCBuffer {
    fn new(buffer: [Section; 9], data_buffer: &[u8]) -> Self {
        let mut data: Vec<u8> = vec![];

        // deconstruct the pc data from the the pc buffers
        for section in buffer {
            if section.read_id(data_buffer) == SectionID::PCbufferI {
                data.extend_from_slice(
                    &data_buffer[section.offset..section.offset + PC_BUFFER_I_SECTION_SIZE],
                );
            } else {
                data.extend_from_slice(
                    &data_buffer[section.offset..section.offset + PC_BUFFER_SECTION_SIZE],
                );
            }
        }

        PCBuffer { buffer, data }
    }

    pub fn get_box(&self, number: usize) -> Vec<Pokemon> {
        let mut boxes = self.data[0x0004..0x8344].chunks(2400);
        let pc = boxes.nth(number).unwrap();
        let mut list: Vec<Pokemon> = vec![];

        for (i, pokemon) in pc.chunks(80).enumerate() {
            // data_offset + pc box offset + slot offset
            let offset = 0x0004 + (number * 2400) + (i * 80);
            let pokemon = Pokemon::new(offset, pokemon);
            list.push(pokemon);
        }

        list
    }

    pub fn read_current_box(&self, buffer: &[u8]) -> u32 {
        LittleEndian::read_u32(&self.buffer[0].read_data(buffer)[..0x0004])
    }

    pub fn write_checksum(&self, buffer: &mut [u8]) {
        for section in self.buffer {
            section.write_checksum(buffer);
        }
    }

    pub fn test1(&self, buffer: &mut [u8]) {
        let mut pc_buffer: Vec<u8> = vec![];

        // deconstruct the pc data from the the pc buffers
        for section in self.buffer {
            if section.read_id(buffer) == SectionID::PCbufferI {
                pc_buffer.extend_from_slice(
                    &buffer[section.offset..section.offset + PC_BUFFER_I_SECTION_SIZE],
                );
            } else {
                pc_buffer.extend_from_slice(
                    &buffer[section.offset..section.offset + PC_BUFFER_SECTION_SIZE],
                );
            }
        }

        /*for pokemon_buffer in pc_buffer[0x0004..0x8344].chunks(80) {
            let pokemon = Pokemon::new(pokemon_buffer);
            let species = pokemon.read_species();

            if species != 0 {
                println!("{}, species: {}", pokemon.read_name(), species);
            }
        }*/
        let mut torchic = vec![0; 80];
        torchic.copy_from_slice(&pc_buffer[0x0004..0x0004 + 80]);

        pc_buffer[0x0004 + 80..0x0004 + 80 + 80].copy_from_slice(&torchic);

        // how to save changes to the pc
        for (i, section) in pc_buffer.chunks(PC_BUFFER_SECTION_SIZE).enumerate() {
            if i == 8 {
                buffer[self.buffer[i].offset..self.buffer[i].offset + PC_BUFFER_I_SECTION_SIZE]
                    .copy_from_slice(&section);
            } else {
                buffer[self.buffer[i].offset..self.buffer[i].offset + PC_BUFFER_SECTION_SIZE]
                    .copy_from_slice(&section);
            }

            self.buffer[i].write_checksum(buffer);
        }
    }
}

/// The Generation III save file is broken up into two game save blocks (Game Save A, Game Save B), each of which is broken up into 14 4KB sections.
pub struct SaveData {
    game_save_a: [Section; NUMBER_GAME_SAVE_SECTIONS],
    game_save_b: [Section; NUMBER_GAME_SAVE_SECTIONS],
    //hall_of_fame: Offset,
}

impl Default for SaveData {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveData {
    pub fn new() -> Self {
        let mut game_save_a: [Section; NUMBER_GAME_SAVE_SECTIONS] =
            [Section::default(); NUMBER_GAME_SAVE_SECTIONS];
        let mut game_save_b: [Section; NUMBER_GAME_SAVE_SECTIONS] =
            [Section::default(); NUMBER_GAME_SAVE_SECTIONS];

        // Read each game save block, the two sets ot 14 game save sections
        for i in 0..NUMBER_GAME_SAVE_SECTIONS {
            let save_a_offset = GAME_SAVE_A_OFFSET + (i * SECTION_SIZE);
            let save_b_offset = GAME_SAVE_B_OFFSET + (i * SECTION_SIZE);

            game_save_a[i] = Section {
                offset: save_a_offset,
                size: SECTION_SIZE,
            };
            game_save_b[i] = Section {
                offset: save_b_offset,
                size: SECTION_SIZE,
            };
        }

        SaveData {
            game_save_a,
            game_save_b,
            /*hall_of_fame: Offset {
                offset: HALL_FAME_OFFSET,
                size: HALL_FAME_SIZE,
            },*/
        }
    }

    /// For Ruby and Sapphire, this value will be 0x00000000.
    /// For FireRed and LeafGreen, this value will be 0x00000001.
    /// For Emerald any value other than 0 or 1 can be used.
    pub fn read_game_code(&self, buffer: &[u8]) -> u32 {
        let section = self.get_section(buffer, SectionID::TrainerInfo).unwrap();
        let section_data_buffer = section.read_data(buffer);
        LittleEndian::read_u32(&section_data_buffer[0x00AC..0x00AC + 4])
    }

    /// The security_key location may vary depending on the game.
    /// --------------------------------------
    /// | Offset | Size | Game |
    /// |   N/A  |  N/A |  RS  |
    /// | 0x00AC |   4  |  RS  |
    /// | 0x0AF8 |   4  | FrLg |
    /// --------------------------------------
    /// Ruby and Sapphire either do not utilize this masking operation, or the mask is always zero.
    pub fn read_security_key(&self, buffer: &[u8]) -> u32 {
        let game_code = self.read_game_code(buffer);

        if game_code == 0x00000000 {
            0x00000000
        } else if game_code == 0x00000001 {
            let section = self.get_section(buffer, SectionID::TrainerInfo).unwrap();
            let section_data_buffer = section.read_data(buffer);
            LittleEndian::read_u32(&section_data_buffer[0x0AF8..0x0AF8 + 4])
        } else {
            game_code
        }
    }

    pub fn read_security_key_lower(&self, buffer: &[u8]) -> u16 {
        LittleEndian::read_u16(&self.read_security_key(buffer).to_le_bytes()[..2])
    }

    /// The amount of money held by the player. Found in the Team/Items section.
    pub fn read_money(&self, buffer: &[u8]) -> u32 {
        let security_key = self.read_security_key(buffer);

        let section = self.get_section(buffer, SectionID::TeamItems).unwrap();
        let section_data_buffer = section.read_data(buffer);

        // Must be XORed with the security key to yield the true value.
        LittleEndian::read_u32(&section_data_buffer[0x0490..0x0490 + 4]) ^ security_key
    }

    pub fn write_money(&self, buffer: &mut [u8], amount: u32) {
        let security_key = self.read_security_key(buffer);

        let section = self.get_section(buffer, SectionID::TeamItems).unwrap();
        let section_data_buffer = section.read_data_mut(buffer);

        // Must be XORed with the security key to yield the true value.
        let money = amount ^ security_key;

        section_data_buffer[0x0490..0x0490 + 4].copy_from_slice(&money.to_le_bytes());

        section.write_checksum(buffer);
    }

    pub fn read_ball_item_pocket<'a>(&'a self, buffer: &'a [u8]) -> &'a [u8] {
        let game_code = self.read_game_code(buffer);
        let section = self.get_section(buffer, SectionID::TeamItems).unwrap();
        let section_data_buffer = section.read_data(buffer);

        if game_code == 0x00000000 {
            &section_data_buffer[0x0430..0x0464]
        } else if game_code == 0x00000001 {
            &section_data_buffer[0x0600..0x0640]
        } else {
            &section_data_buffer[0x0650..0x0690]
        }
    }

    fn write_ball_item_pocket<'a>(&'a self, buffer: &'a mut [u8]) -> &'a mut [u8] {
        let game_code = self.read_game_code(buffer);
        let section = self.get_section(buffer, SectionID::TeamItems).unwrap();
        let section_data_buffer = section.read_data_mut(buffer);

        if game_code == 0x00000000 {
            &mut section_data_buffer[0x0430..0x0464]
        } else if game_code == 0x00000001 {
            &mut section_data_buffer[0x0600..0x0640]
        } else {
            &mut section_data_buffer[0x0650..0x0690]
        }
    }

    pub fn add_to_ball_pocket(
        &self,
        buffer: &mut [u8],
        ball_id: u16,
        quantity: u16,
    ) -> Result<(), DataStructureError> {
        if ball_id > 12 && ball_id != 0 {
            return Err(DataStructureError::InvalidIndex);
        }

        let security_key = self.read_security_key_lower(buffer);
        let balls_pokect = self.write_ball_item_pocket(buffer);
        let mut added = false;

        for item_entry in balls_pokect.chunks_mut(4) {
            let id = LittleEndian::read_u16(&item_entry[..2]);

            if !added && id == 0 {
                item_entry[0x00..0x02].copy_from_slice(&ball_id.to_le_bytes());
                LittleEndian::write_u16(&mut item_entry[0x02..], quantity ^ security_key);
                added = true;
            }
        }

        let section = self.get_section(buffer, SectionID::TeamItems).unwrap();
        section.write_checksum(buffer);

        Ok(())
    }

    fn write_item_pocket<'a>(&'a self, buffer: &'a mut [u8]) -> &'a mut [u8] {
        let game_code = self.read_game_code(buffer);
        let section = self.get_section(buffer, SectionID::TeamItems).unwrap();
        let section_data_buffer = section.read_data_mut(buffer);

        if game_code == 0x00000000 {
            &mut section_data_buffer[0x0560..0x05B0]
        } else if game_code == 0x00000001 {
            &mut section_data_buffer[0x0310..0x03B8]
        } else {
            &mut section_data_buffer[0x0560..0x05D8]
        }
    }

    pub fn add_to_item_pocket(
        &self,
        buffer: &mut [u8],
        item_id: u16,
        quantity: u16,
    ) -> Result<(), DataStructureError> {
        // TODO: item_id validation

        let security_key = self.read_security_key_lower(buffer);
        let item_pokect = self.write_item_pocket(buffer);
        let mut added = false;

        for item_entry in item_pokect.chunks_mut(4) {
            let id = LittleEndian::read_u16(&item_entry[..2]);

            if !added && id == 0 {
                item_entry[0x00..0x02].copy_from_slice(&item_id.to_le_bytes());
                LittleEndian::write_u16(&mut item_entry[0x02..], quantity ^ security_key);
                added = true;
            }
        }

        let section = self.get_section(buffer, SectionID::TeamItems).unwrap();
        section.write_checksum(buffer);

        Ok(())
    }

    pub fn read_team_list(&self, buffer: &[u8]) -> Vec<Pokemon> {
        let game_code = self.read_game_code(buffer);
        let section = self.get_section(buffer, SectionID::TeamItems).unwrap();
        let section_data_buffer = section.read_data(buffer);

        let mut team: Vec<Pokemon> = vec![];

        if game_code == 0x00000001 {
            for (i, pokemon_data) in section_data_buffer[0x0038..0x0290].chunks(100).enumerate() {
                let offset = section.get_offset() + 0x0038 + (i * 100);
                let pokemon = Pokemon::new(offset, &pokemon_data);
                //let pokemon = Pokemon::from_offset(offset, &buffer);
                team.push(pokemon);
            }
        } else {
            for (i, pokemon_data) in section_data_buffer[0x0238..0x0490].chunks(100).enumerate() {
                let offset = section.get_offset() + 0x0238 + (i * 100);
                let pokemon = Pokemon::new(offset, &pokemon_data);
                //let pokemon = Pokemon::from_offset(offset, &buffer);
                team.push(pokemon);
            }
        }

        team
    }

    fn write_team_list<'a>(&'a self, buffer: &'a mut [u8]) -> &'a mut [u8] {
        let game_code = self.read_game_code(buffer);
        let section = self.get_section(buffer, SectionID::TeamItems).unwrap();
        let section_data_buffer: &mut [u8] = section.read_data_mut(buffer);

        if game_code == 0x00000001 {
            &mut section_data_buffer[0x0038..0x0290]
        } else {
            &mut section_data_buffer[0x0238..0x0490]
        }
    }

    pub fn add_to_team(&self, buffer: &mut [u8], pokemon: &[u8]) {
        let team_list = self.write_team_list(buffer);
        let mut added = false;

        for slot in team_list.chunks_mut(100) {
            if slot[0] == 0 && !added {
                slot.copy_from_slice(pokemon);
                added = true;
            }
        }

        if !added {
            team_list[500..].copy_from_slice(pokemon);
        }

        let section = self.get_section(buffer, SectionID::TeamItems).unwrap();
        section.write_checksum(buffer);
    }

    fn get_section(&self, buffer: &[u8], id: SectionID) -> Option<&Section> {
        let current_save = self.current_save(buffer);

        current_save
            .iter()
            .find(|section| section.read_id(buffer) == id)
    }

    pub fn get_pc_buffer<'a>(&'a self, buffer: &'a [u8]) -> PCBuffer {
        let current_save = self.current_save(buffer);

        let range = 5..=13;

        // Read all the Sections that hold the pc buffer from PCBufferA to PCbufferI
        let mut sections: Vec<&Section> = current_save
            .iter()
            .filter(|section| range.contains(&section.read_id(buffer).into()))
            .collect();

        sections.sort_by(|a, b| a.read_id(buffer).partial_cmp(&b.read_id(buffer)).unwrap());

        let sections: Vec<Section> = sections.iter().map(|section| **section).collect();

        // Store them into an array for easy access
        let mut pc_buffer: [Section; 9] = [Section::default(); 9];

        pc_buffer.copy_from_slice(sections.as_slice());

        PCBuffer::new(pc_buffer, buffer)
    }

    fn current_save(&self, buffer: &[u8]) -> &[Section] {
        let save_index_a = self.game_save_a[0].read_save_index(buffer);
        let save_index_b = self.game_save_b[0].read_save_index(buffer);

        if save_index_a == u32::MAX {
            return &self.game_save_b;
        }
        if save_index_a > save_index_b {
            return &self.game_save_a;
        }

        &self.game_save_b
    }
}
