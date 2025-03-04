//! Implementation of [Pokemon Gen (III) Save file data structure](https://bulbapedia.bulbagarden.net/wiki/Save_data_structure_(Generation_III)).
//!
//! The structure consists of 128 KB of data, though not every byte is used. The integrity of most of the file is validated by checksums.
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
use crate::misc::{find_item, item_id_g3};

/// Represents errors that can occur while handling save data.
#[derive(Error, Debug)]
pub enum SaveDataError {
    /// Section not found by ID
    #[error("Section not found for ID {0:?}")]
    SectionNotFound(SectionID),

    /// Invalid data length encountered
    #[error("Invalid data length: expected {expected}, found {found}")]
    InvalidDataLength { expected: usize, found: usize },

    /// Invalid offset or out-of-bounds access
    #[error("Invalid offset: {0}")]
    InvalidOffset(usize),

    /// Decryption failure
    #[error("Decryption failed for key {0:#X}")]
    DecryptionError(u16),

    /// Checksum mismatch detected
    #[error("Checksum mismatch: expected {expected:#X}, found {found:#X}")]
    ChecksumMismatch { expected: u16, found: u16 },

    /// Unexpected error occurred
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

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
#[derive(Default, Debug, Clone)]
pub struct SaveFile {
    game_save_a: [Section; NUMBER_GAME_SAVE_SECTIONS],
    game_save_b: [Section; NUMBER_GAME_SAVE_SECTIONS],
    data: Vec<u8>,
    pc_buffer: PCBuffer,
}

pub enum Pocket {
    Items,
    Pokeballs,
    Berries,
    Tms,
    Key,
}

fn pocket_address(pocket: Pocket, game_code: u32) -> (usize, usize) {
    // For Ruby and Sapphire, this value will be 0x00000000.
    // For FireRed and LeafGreen, this value will be 0x00000001.
    // For Emerald any value other than 0 or 1 can be used.
    // Determine offsets dynamically based on game version
    match pocket {
        Pocket::Items => match game_code {
            0x00000000 => (0x0560, 0x05B0), // Ruby/Sapphire
            0x00000001 => (0x0310, 0x03B8), // FireRed/LeafGreen
            _ => (0x0560, 0x05D8),          // Emerald
        },
        Pocket::Pokeballs => match game_code {
            0x00000000 => (0x0600, 0x0640), // Ruby/Sapphire
            0x00000001 => (0x0430, 0x0464), // FireRed/LeafGreen
            _ => (0x0650, 0x0690),          // Emerald
        },
        Pocket::Berries => match game_code {
            0x00000000 => (0x0740, 0x7F8), // Ruby/Sapphire
            0x00000001 => (0x054C, 0x5F8), // FireRed/LeafGreen
            _ => (0x0790, 0x848),          // Emerald
        },
        Pocket::Tms => match game_code {
            0x00000000 => (0x0640, 0x0740), // Ruby/Sapphire
            0x00000001 => (0x0464, 0x054C), // FireRed/LeafGreen
            _ => (0x0690, 0x0790),          // Emerald
        },
        Pocket::Key => match game_code {
            0x00000000 => (0x05B0, 0x0600), // Ruby/Sapphire
            0x00000001 => (0x03B8, 0x0430), // FireRed/LeafGreen
            _ => (0x05D8, 0x0650),          // Emerald
        },
    }
}

impl SaveFile {
    pub fn new(data: &[u8]) -> Self {
        let mut game_save_a: [Section; NUMBER_GAME_SAVE_SECTIONS] =
            [Section::default(); NUMBER_GAME_SAVE_SECTIONS];
        let mut game_save_b: [Section; NUMBER_GAME_SAVE_SECTIONS] =
            [Section::default(); NUMBER_GAME_SAVE_SECTIONS];

        // Read each game save block, the two sets of 14 game save sections
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

    pub fn is_empty(&self) -> bool {
        self.data.len() == 0
    }

    pub fn ot_name(&self) -> Vec<u8> {
        let section = self
            .get_section(SectionID::TrainerInfo)
            .expect("Expected value but found None");
        let section_data_buffer = section.data(&self.data);

        section_data_buffer[0x0000..7].to_vec()
    }

    pub fn ot_id(&self) -> Vec<u8> {
        let section = self
            .get_section(SectionID::TrainerInfo)
            .expect("Expected value but found None");
        let section_data_buffer = section.data(&self.data);

        section_data_buffer[0x000A..0x000A + 4].to_vec()
    }

    pub fn get_party(&self) -> Result<Vec<Pokemon>, SaveDataError> {
        let game_code = self.get_game_code()?;
        let section = self
            .get_section(SectionID::TeamItems)
            .expect("Expected value but found None");
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

        Ok(team)
    }

    pub fn pc_box(&self, number: usize) -> Vec<Pokemon> {
        self.pc_buffer.pc_box(number)
    }

    pub fn is_pc_empty(&self) -> bool {
        self.pc_buffer.is_empty()
    }

    pub fn save_pokemon(
        &mut self,
        storage: StorageType,
        pokemon: Pokemon,
    ) -> Result<(), SaveDataError> {
        match storage {
            StorageType::Party => {
                let offset = pokemon.offset();

                self.data[offset..offset + 100].copy_from_slice(&pokemon.raw_data());
                let section = self
                    .get_section(SectionID::TeamItems)
                    .expect("Expected value but found None");
                section.write_checksum(&mut self.data)?;
            }
            StorageType::PC => {
                self.pc_buffer.save_pokemon(pokemon, &mut self.data)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// For Ruby and Sapphire, this value will be 0x00000000.
    /// For FireRed and LeafGreen, this value will be 0x00000001.
    /// For Emerald any value other than 0 or 1 can be used.
    pub fn game_code(&self) -> u32 {
        let section = self
            .get_section(SectionID::TrainerInfo)
            .expect("Expected value but found None");
        let section_data_buffer = section.data(&self.data);
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
    fn security_key(&self) -> u32 {
        let game_code = self.game_code();

        if game_code == 0x00000000 {
            0x00000000
        } else if game_code == 0x00000001 {
            let section = self
                .get_section(SectionID::TrainerInfo)
                .expect("Expected value but found None");
            let section_data_buffer = section.data(&self.data);
            LittleEndian::read_u32(&section_data_buffer[0x0AF8..0x0AF8 + 4])
        } else {
            game_code
        }
    }

    fn security_key_lower(&self) -> u16 {
        LittleEndian::read_u16(&self.security_key().to_le_bytes()[..2])
    }

    /// Retrieves the pocket data from the save file.
    ///
    /// Offsets and data encryption vary depending on the game version.
    pub fn pocket(&self, pocket: Pocket) -> Result<Vec<(String, u16)>, SaveDataError> {
        let game_code = self.game_code();
        let (start, end) = pocket_address(pocket, game_code);
        self.read_pocket(start, end)
    }

    /// Saves the updated pocket data back into the save file.
    ///
    /// This function writes the modified pocket data into the corresponding save section,
    /// encrypting it with the security key.
    pub fn save_pocket(&mut self, pocket_type: Pocket, pocket_list: Vec<(String, u16)>) -> Result<(), SaveDataError> {
        let game_code = self.game_code();
        let security_key = self.security_key_lower();
        let (start, end) = pocket_address(pocket_type, game_code);

        let section = self
            .get_section(SectionID::TeamItems)
            .ok_or(SaveDataError::SectionNotFound(SectionID::TeamItems))?;
        let section_data_buffer: &mut [u8] = section.data_mut(&mut self.data);

        let encrypted_bag = SaveFile::encrypt_pocket(pocket_list, security_key)?;
        section_data_buffer[start..end].copy_from_slice(&encrypted_bag);
        section.write_checksum(&mut self.data)?;

        Ok(())
    }

    fn read_pocket(&self, start: usize, end: usize) -> Result<Vec<(String, u16)>, SaveDataError> {
        let security_key = self.security_key_lower();

        let section = self
            .get_section(SectionID::TeamItems)
            .ok_or(SaveDataError::SectionNotFound(SectionID::TeamItems))?;
        let section_data_buffer = section.data(&self.data);

        let bag = SaveFile::decrypt_pocket(&section_data_buffer[start..end], security_key)?;

        Ok(bag)
    }

    /// Helper function to decrypt pocket data using the security key.
    ///
    /// Each pocket entry consists of:
    /// - First 2 bytes: Item ID (u16)
    /// - Last 2 bytes: Quantity (u16, XORed with the security key)
    fn decrypt_pocket(data: &[u8], security_key: u16) -> Result<Vec<(String, u16)>, SaveDataError> {
        let mut pocket = Vec::new();

        for chunk in data.chunks(4) {
            if chunk.len() < 4 {
                return Err(SaveDataError::InvalidDataLength {
                    expected: 4,
                    found: chunk.len(),
                });
            }

            let item_id = LittleEndian::read_u16(&chunk[0..2]);
            let encrypted_quantity = LittleEndian::read_u16(&chunk[2..4]);
            let quantity = encrypted_quantity ^ security_key;

            let item_name = find_item(item_id as usize).unwrap_or_else(|_| "Unknown".to_string());
            pocket.push((item_name, quantity));
        }

        Ok(pocket)
    }

    /// Helper function to encrypt pocket data using the security key.
    ///
    /// # Arguments
    /// - `pocket`: A vector of tuples containing the item name and quantity.
    /// - `security_key`: The security key used for encryption.
    ///
    /// # Returns
    /// A byte vector containing the encrypted pocket data.
    fn encrypt_pocket(
        pocket: Vec<(String, u16)>,
        security_key: u16,
    ) -> Result<Vec<u8>, SaveDataError> {
        let mut encrypted_data = Vec::new();

        for (item_name, quantity) in pocket {
            let item_id = item_id_g3(&item_name).unwrap_or(0);
            let encrypted_quantity = quantity ^ security_key;

            encrypted_data.extend(&item_id.to_le_bytes());
            encrypted_data.extend(&encrypted_quantity.to_le_bytes());
        }

        Ok(encrypted_data)
    }

    pub fn raw_data(&self) -> Vec<u8> {
        self.data.to_vec()
    }

    fn init_pc_buffer(&mut self) {
        let current_save = self.current_save();

        let range = 5..=13;

        // Read all the Sections that hold the pc buffer from PCBufferA to PCbufferI
        let mut sections: Vec<&Section> = current_save
            .iter()
            .filter(|section| range.contains(&section.id(&self.data).into()))
            .collect();

        sections.sort_by(|a, b| {
            a.id(&self.data)
                .partial_cmp(&b.id(&self.data))
                .expect("Expected value but found None")
        });

        let sections: Vec<Section> = sections.iter().map(|section| **section).collect();

        // Store them into an array for easy access
        let mut pc_buffer: [Section; 9] = [Section::default(); 9];

        pc_buffer.copy_from_slice(sections.as_slice());

        self.pc_buffer = PCBuffer::new(pc_buffer, &self.data);
    }

    /// Determines the most recent save block (A or B) based on the save index.
    ///
    /// Each section in the save file contains a save index, but only the index in the last section is
    /// considered when determining the most recent save. The save index increases every time the game
    /// is saved, even when starting a new game.
    ///
    /// Returns a slice of sections corresponding to the most recent save block.
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

    /// Retrieves the game code, which identifies the version of the game (e.g., Ruby, Sapphire, Emerald).
    ///
    /// The game code is stored in the Trainer Info section. For example:
    /// - `0x00000000`: Ruby/Sapphire
    /// - `0x00000001`: FireRed/LeafGreen
    /// - Any other value: Emerald
    ///
    /// Returns the game code or an error if the Trainer Info section is missing.
    fn get_game_code(&self) -> Result<u32, SaveDataError> {
        let section = self
            .get_section(SectionID::TrainerInfo)
            .ok_or(SaveDataError::SectionNotFound(SectionID::TrainerInfo))?;
        let section_data_buffer = section.data(&self.data);
        Ok(LittleEndian::read_u32(
            &section_data_buffer[0x00AC..0x00AC + 4],
        ))
    }

    fn get_section(&self, id: SectionID) -> Option<Section> {
        let current_save = self.current_save();

        current_save
            .iter()
            .find(|section| section.id(&self.data) == id)
            .copied()
    }
}

/// The Pokémon save file is divided into 14 sections, each corresponding to a specific aspect of the game.
/// These sections include Trainer Info, Items, PC Box Data, etc.
///
/// This struct provides methods for accessing and modifying the save file's sections, managing checksums,
/// and ensuring data integrity.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Default)]
struct Section {
    offset: usize,
    size: usize,
}

impl Section {
    /// Retrieves the save index for this section.
    /// Every time the game is saved, its Save Index value goes up by one. This is true even when starting a new game: it continues to count up from the previous save. All 14 sections within a game save must have the same Save Index value. The most recent game save will have a greater Save Index value than the previous save.
    fn save_index(&self, buffer: &[u8]) -> u32 {
        let section_buffer = &buffer[self.offset..self.offset + self.size];
        LittleEndian::read_u32(&section_buffer[0x0FFC..])
    }

    /// Retrieves the section ID, which identifies the section's purpose (e.g., Trainer Info, PC Buffer A).
    fn id(&self, buffer: &[u8]) -> SectionID {
        let section_buffer = &buffer[self.offset..self.offset + self.size];
        let id = LittleEndian::read_u16(&section_buffer[0x0FF4..0x0FF6]);
        id.into()
    }

    /// Reads the data of the section.
    fn data<'a>(&'a self, buffer: &'a [u8]) -> &'a [u8] {
        let section_buffer = &buffer[self.offset..self.offset + self.size];
        &section_buffer[0..SECTION_DATA_SIZE]
    }

    /// Retrieves mutable data of the section.
    fn data_mut<'a>(&'a self, buffer: &'a mut [u8]) -> &'a mut [u8] {
        let section_buffer = &mut buffer[self.offset..self.offset + self.size];
        &mut section_buffer[0..SECTION_DATA_SIZE]
    }

    fn offset(&self) -> usize {
        self.offset
    }

    /// Updates the checksum of the section to reflect changes in the data.
    /// Used to validate the integrity of saved data.
    /// A 16-bit checksum generated by adding up bytes from the section. The algorithm is as follows:
    /// -Initialize a 32-bit checksum variable to zero.
    /// -Read 4 bytes at a time as 32-bit word (little-endian) and add it to the variable.
    /// -Take the upper 16 bits of the result, and add them to the lower 16 bits of the result.
    /// -This new 16-bit value is the checksum.
    fn write_checksum(&self, buffer: &mut [u8]) -> Result<(), SaveDataError> {
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

        Ok(())
    }
}

/// Representation of the PC Buffer.
///
/// In Pokémon Generation III games, the save file is broken into two save blocks (Save A and Save B), each consisting of 14 sections. The PC Buffer spans multiple sections and contains the stored Pokémon data.
#[derive(Default, Debug, Clone)]
pub struct PCBuffer {
    /// The sections that collectively store the PC data.
    buffer: [Section; 9],
    /// The combined data extracted from the sections for easier processing.
    data: Vec<u8>,
}

impl PCBuffer {
    /// Creates a new PCBuffer by combining data from the specified sections.
    /// The PCBuffer is constructed by extracting data from the sections that contain the PC data.
    /// Special handling is required for the last section (`PCbufferI`), which may have a different size.
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

    /// Retrieves all Pokémon stored in a specific PC box.
    /// Each PC box is a fixed-size chunk of the PC Buffer, containing 30 Pokémon slots.
    fn pc_box(&self, number: usize) -> Vec<Pokemon> {
        let mut boxes = self.data[0x0004..0x8344].chunks(2400);
        let pc = boxes.nth(number).expect("Expected value but found None");
        let mut list: Vec<Pokemon> = vec![];

        for (i, pokemon) in pc.chunks(80).enumerate() {
            // data_offset + pc box offset + slot offset
            let offset = 0x0004 + (number * 2400) + (i * 80);
            let pokemon = Pokemon::new(offset, pokemon);
            list.push(pokemon);
        }

        list
    }

    /// Saves a Pokémon back into the PC Buffer and updates the relevant sections.
    ///
    /// # Arguments
    /// - `pokemon`: The Pokémon to save.
    /// - `buffer`: The save file data buffer to update.
    fn save_pokemon(&mut self, pokemon: Pokemon, buffer: &mut [u8]) -> Result<(), SaveDataError> {
        let offset = pokemon.offset();
        self.data[offset..offset + 80].copy_from_slice(&pokemon.raw_data()[..80]);

        // Update each section of the PC Buffer and recalculate checksums.
        for (i, section) in self.data.chunks(PC_BUFFER_SECTION_SIZE).enumerate() {
            if i == 8 {
                buffer[self.buffer[i].offset..self.buffer[i].offset + PC_BUFFER_I_SECTION_SIZE]
                    .copy_from_slice(section);
            } else {
                buffer[self.buffer[i].offset..self.buffer[i].offset + PC_BUFFER_SECTION_SIZE]
                    .copy_from_slice(section);
            }

            self.buffer[i].write_checksum(buffer)?;
        }

        Ok(())
    }

    /// Checks if the PC Buffer is empty.
    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Represents the player's internal Trainer ID.
///
/// The Trainer ID is split into two components:
/// - The **public ID** (lower 16 bits), which is visible in-game.
/// - The **private ID** (upper 16 bits), which is used internally for certain mechanics (e.g., shiny Pokémon).
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

impl Into<Vec<u8>> for TrainerID {
    fn into(self) -> Vec<u8> {
        let buffer: Vec<u8> = vec![0, 0, 0, 0];

        buffer
    }
}

/// Enum representing the ID of a save file section.
/// Specifies the save data being represented
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Default)]
pub enum SectionID {
    #[default]
    TrainerInfo,
    TeamItems,
    GameState,
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
            2 => SectionID::GameState,
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
            SectionID::GameState => 2,
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

#[derive(Debug, Copy, Clone, Default)]
pub enum StorageType {
    PC,
    Party,
    #[default]
    None,
}
