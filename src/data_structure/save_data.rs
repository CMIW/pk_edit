//! Implementation of [Pokemon Gen (III) Save file data structure](https://bulbapedia.bulbagarden.net/wiki/Save_data_structure_(Generation_III)).
//!
//! The structure consists of 128 KB of data, though not every byte is used. When emulated, this data is generally placed in a separate file (".sav" is a common extension). The integrity of most of the file is validated by checksums.
//!
//! # File structure
//!
//! The Generation III save file is broken up into two game save blocks (Game Save A, Game Save B), each of which is broken up into 14 4KB sections.
//!
//! One block of game save data represents the most recent game save, and the other block represents the previous game save.
//!
//!|  Offset  | Size  |   Contents    |
//!|----------|-------|---------------|
//!| 0x000000 | 57344 |  Game save A  |
//!| 0x00E000 | 57344 |  Game save B  |
//!
//! # Section format
//!
//! All sections contain the same general format: 3968 bytes of data, followed by a footer. All 14 sections must be present exactly once in each game save block.
//!
//!| Offset | Size | Contents    |
//!|--------|------|-------------|
//!| 0x0000 | 3968 |  Data       |
//!| 0x0FF4 |   2  |  Section ID |
//!| 0x0FF6 |   2  |  Checksum   |
//!| 0x0FF8 |   4  |  Signature  |
//!| 0x0FFC |   4  |  Save Index |
//!
//! Every time the game is saved the order of the sections gets rotated once.The sections only ever get rotated and never scrambled so they will always remain in the same order, but may begin at different points.
//!
//! ## Checksum
//!
//! Used to validate the integrity of saved data.
//! A 16-bit checksum generated by adding up bytes from the section. The algorithm is as follows:
//! -Initialize a 32-bit checksum variable to zero.
//! -Read 4 bytes at a time as 32-bit word (little-endian) and add it to the variable.
//! -Take the upper 16 bits of the result, and add them to the lower 16 bits of the result.
//! -This new 16-bit value is the checksum.
//!
//! ## Save Index
//!
//! Every time the game is saved, its Save Index value goes up by one. This is true even when starting a new game: it continues to count up from the previous save. All 14 sections within a game save must have the same Save Index value. The most recent game save will have a greater Save Index value than the previous save.
//!
//! # Section ID
//!
//!| ID | Size |    Contents    |
//!|----|------|----------------|
//!| 00 | 3884 |  Trainer info  |
//!| 01 | 3968 |  Team / items  |
//!| 02 | 3968 |   Game State   |
//!| 03 | 3968 |   Misc Data    |
//!| 04 | 3968 |   Rival info   |
//!| 05 | 3968 |  PC buffer A   |
//!| 06 | 3968 |  PC buffer A   |
//!| 07 | 3968 |  PC buffer A   |
//!| 08 | 3968 |  PC buffer A   |
//!| 09 | 3968 |  PC buffer A   |
//!| 10 | 3968 |  PC buffer A   |
//!| 11 | 3968 |  PC buffer A   |
//!| 12 | 3968 |  PC buffer A   |
//!| 13 | 2000 |  PC buffer A   |
//!
use byteorder::{ByteOrder, LittleEndian};
use std::convert::From;
use std::default::Default;
use thiserror::Error;

use crate::data_structure::pokemon::Pokemon;

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

/// Representation of the Save File.
///
/// The Generation III save file is broken up into two game save blocks (Game Save A, Game Save B), each of which is broken up into 14 4KB sections.
#[derive(Default)]
pub struct SaveFile {
    game_save_a: [Section; NUMBER_GAME_SAVE_SECTIONS],
    game_save_b: [Section; NUMBER_GAME_SAVE_SECTIONS],
    data: Vec<u8>,
    pc_buffer: PCBuffer,
}

impl SaveFile {
    pub fn new(data: &[u8]) -> Self {
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

        let mut save = SaveFile {
            game_save_a,
            game_save_b,
            data: data.to_vec(),
            pc_buffer: PCBuffer::default(),
        };

        save.init_pc_buffer();

        save
    }

    pub fn get_party(&self) -> Vec<Pokemon> {
        let game_code = self.get_game_code();
        let section = self.get_section(SectionID::TeamItems).unwrap();
        let section_data_buffer = section.data(&self.data);

        let mut team: Vec<Pokemon> = vec![];

        if game_code == 0x00000001 {
            for (i, pokemon_data) in section_data_buffer[0x0038..0x0290].chunks(100).enumerate() {
                let offset = section.offset() + 0x0038 + (i * 100);
                let pokemon = Pokemon::new(offset, pokemon_data);
                team.push(pokemon);
            }
        } else {
            for (i, pokemon_data) in section_data_buffer[0x0238..0x0490].chunks(100).enumerate() {
                let offset = section.offset() + 0x0238 + (i * 100);
                let pokemon = Pokemon::new(offset, pokemon_data);
                team.push(pokemon);
            }
        }

        team
    }

    pub fn pc_box(&self, number: usize) -> Vec<Pokemon> {
        self.pc_buffer.pc_box(number)
    }

    pub fn is_pc_empty(&self) -> bool {
        self.pc_buffer.is_empty()
    }

    fn init_pc_buffer(&mut self) {
        let current_save = self.current_save();

        let range = 5..=13;

        // Read all the Sections that hold the pc buffer from PCBufferA to PCbufferI
        let mut sections: Vec<&Section> = current_save
            .iter()
            .filter(|section| range.contains(&section.id(&self.data).into()))
            .collect();

        sections.sort_by(|a, b| a.id(&self.data).partial_cmp(&b.id(&self.data)).unwrap());

        let sections: Vec<Section> = sections.iter().map(|section| **section).collect();

        // Store them into an array for easy access
        let mut pc_buffer: [Section; 9] = [Section::default(); 9];

        pc_buffer.copy_from_slice(sections.as_slice());

        self.pc_buffer = PCBuffer::new(pc_buffer, &self.data);
    }

    /// Despite the save index being stored in each section, only the value in the last section is used to determine the most recent save. If save A's value is bigger, then it is the most recent. Otherwise, save B is the most recent (this includes ties).
    fn current_save(&self) -> &[Section] {
        let save_index_a = self.game_save_a[0].save_index(&self.data);
        let save_index_b = self.game_save_b[0].save_index(&self.data);

        if save_index_a == u32::MAX {
            return &self.game_save_b;
        }
        if save_index_a > save_index_b {
            return &self.game_save_a;
        }

        &self.game_save_b
    }

    /// For Ruby and Sapphire, this value will be 0x00000000.
    /// For FireRed and LeafGreen, this value will be 0x00000001.
    /// For Emerald any value other than 0 or 1 can be used.
    fn get_game_code(&self) -> u32 {
        let section = self.get_section(SectionID::TrainerInfo).unwrap();
        let section_data_buffer = section.data(&self.data);
        LittleEndian::read_u32(&section_data_buffer[0x00AC..0x00AC + 4])
    }

    fn get_section(&self, id: SectionID) -> Option<&Section> {
        let current_save = self.current_save();

        current_save
            .iter()
            .find(|section| section.id(&self.data) == id)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Default)]
struct Section {
    offset: usize,
    size: usize,
}

impl Section {
    /// Every time the game is saved, its Save Index value goes up by one. This is true even when starting a new game: it continues to count up from the previous save. All 14 sections within a game save must have the same Save Index value. The most recent game save will have a greater Save Index value than the previous save.
    fn save_index(&self, buffer: &[u8]) -> u32 {
        let section_buffer = &buffer[self.offset..self.offset + self.size];
        LittleEndian::read_u32(&section_buffer[0x0FFC..])
    }

    fn id(&self, buffer: &[u8]) -> SectionID {
        let section_buffer = &buffer[self.offset..self.offset + self.size];
        let id = LittleEndian::read_u16(&section_buffer[0x0FF4..0x0FF6]);
        id.into()
    }

    fn data<'a>(&'a self, buffer: &'a [u8]) -> &'a [u8] {
        let section_buffer = &buffer[self.offset..self.offset + self.size];
        &section_buffer[0..SECTION_DATA_SIZE]
    }

    fn offset(&self) -> usize {
        self.offset
    }

    /// Used to validate the integrity of saved data.
    /// A 16-bit checksum generated by adding up bytes from the section. The algorithm is as follows:
    /// -Initialize a 32-bit checksum variable to zero.
    /// -Read 4 bytes at a time as 32-bit word (little-endian) and add it to the variable.
    /// -Take the upper 16 bits of the result, and add them to the lower 16 bits of the result.
    /// -This new 16-bit value is the checksum.
    fn write_checksum(&self, buffer: &mut [u8]) {
        let mut checksum: u32 = 0;
        let data = self.data(buffer);

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

/// Representation of the PC Buffer.
///
/// The Generation III save file is broken up into two game save blocks (Game Save A, Game Save B), each of which is broken up into 14 4KB sections.
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
            if section.id(data_buffer) == SectionID::PCbufferI {
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

    fn pc_box(&self, number: usize) -> Vec<Pokemon> {
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

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn write_checksum(&self, buffer: &mut [u8]) {
        for section in self.buffer {
            section.write_checksum(buffer);
        }
    }
}

/// The player's internal Trainer ID.
///
/// The lower 16 bits represent the visible, public ID.
/// The upper 16 bits represent the hidden, Secret ID.
#[derive(Debug, Copy, Clone)]
pub struct TrainerID {
    public: u16,
    private: u16,
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
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Default)]
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
