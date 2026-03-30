//! Storage location types and bag-pocket helpers.
//!
//! [`StorageType`] identifies where a [`crate::pokemon::Pokemon`] lives (party, PC, or unset).
//! [`Pocket`] identifies the five bag pockets. [`pocket_address`] maps a pocket to its
//! byte range within Section 1, accounting for game-version differences.
//!
//! Pocket data in save files is XOR-encrypted with the lower 16 bits of the security key.
//! [`decrypt_pocket`] and [`encrypt_pocket`] handle the conversion to/from `(name, quantity)` pairs.

use crate::save::SectionID;
use byteorder::{ByteOrder, LittleEndian};
use std::fmt;

use crate::error::SaveDataError;
use crate::misc::{find_item, item_id_g3};

pub const TEAM_SECTION_ID: SectionID = SectionID::TeamItems;
pub const PARTY_COUNT_OFFSET: usize = 0x0034;
pub const PARTY_DATA_OFFSET: usize = 0x0038;
pub const PARTY_SIZE: usize = 6;
pub const PARTY_POKEMON_SIZE: usize = 100;

/// Where a selected Pokémon is stored within the save file.
#[derive(Debug, Copy, Clone, Default)]
pub enum StorageType {
    /// The Pokémon resides in a PC box (80 bytes stored).
    PC,
    /// The Pokémon is in the active party (100 bytes stored).
    Party,
    /// No Pokémon is currently selected.
    #[default]
    None,
}

/// One of the five bag pockets in a Gen III save file.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Pocket {
    /// Regular items (Potions, Repels, etc.).
    Items,
    /// Pokéballs.
    Pokeballs,
    /// Berries.
    Berries,
    /// TMs and HMs.
    Tms,
    /// Key Items (cannot be discarded).
    Key,
}

impl fmt::Display for Pocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pocket::Items => write!(f, "Items"),
            Pocket::Pokeballs => write!(f, "Poké Balls"),
            Pocket::Berries => write!(f, "Berries"),
            Pocket::Tms => write!(f, "TMs & HMs"),
            Pocket::Key => write!(f, "Key Items"),
        }
    }
}

/// Helper to determine the memory offset range for a specific pocket based on the Game Code.
/// Returns (start_offset, end_offset) within Section 1.
pub fn pocket_address(pocket: Pocket, game_code: u32) -> (usize, usize) {
    // 0 = Ruby/Sapphire, 1 = FireRed/LeafGreen, Others = Emerald
    match pocket {
        Pocket::Items => match game_code {
            0 => (0x0560, 0x05B0),
            1 => (0x0310, 0x03B8),
            _ => (0x0560, 0x05D8),
        },
        Pocket::Pokeballs => match game_code {
            0 => (0x0600, 0x0640),
            1 => (0x0430, 0x0464),
            _ => (0x0650, 0x0690),
        },
        Pocket::Berries => match game_code {
            0 => (0x0740, 0x07F8),
            1 => (0x054C, 0x05F8),
            _ => (0x0790, 0x0848),
        },
        Pocket::Tms => match game_code {
            0 => (0x0640, 0x0740),
            1 => (0x0464, 0x054C),
            _ => (0x0690, 0x0790),
        },
        Pocket::Key => match game_code {
            0 => (0x05B0, 0x0600),
            1 => (0x03B8, 0x0430),
            _ => (0x05D8, 0x0650),
        },
    }
}

/// Decrypts pocket data using the security key.
/// Returns a list of (ItemName, Quantity).
pub fn decrypt_pocket(data: &[u8], security_key: u16) -> Result<Vec<(String, u16)>, SaveDataError> {
    let mut pocket = Vec::new();

    for chunk in data.chunks(4) {
        if chunk.len() < 4 {
            // If padding bytes remain that aren't a full chunk, ignore or error?
            // Usually pockets are aligned to 4 bytes.
            if chunk.iter().all(|&x| x == 0) {
                continue;
            }
            return Err(SaveDataError::InvalidDataLength {
                expected: 4,
                found: chunk.len(),
            });
        }

        let item_id = LittleEndian::read_u16(&chunk[0..2]);
        let encrypted_quantity = LittleEndian::read_u16(&chunk[2..4]);

        // Apply Security Key (XOR)
        let quantity = encrypted_quantity ^ security_key;

        if item_id != 0 {
            // Safe DB lookup
            let item_name = find_item(item_id as usize)
                .unwrap_or_else(|_| format!("Unknown Item ({})", item_id));
            pocket.push((item_name, quantity));
        }
    }

    Ok(pocket)
}

/// Encrypts pocket data using the security key for saving.
/// Returns raw bytes ready to be written to Section 1.
pub fn encrypt_pocket(
    pocket: Vec<(String, u16)>,
    security_key: u16,
) -> Result<Vec<u8>, SaveDataError> {
    let mut encrypted_data = Vec::new();

    for (item_name, quantity) in pocket {
        // Strict or Safe DB lookup?
        // Usually strict for writing, but we default to 0 if not found to prevent crashing logic loops
        let item_id = item_id_g3(&item_name).unwrap_or(0);

        let encrypted_quantity = quantity ^ security_key;

        encrypted_data.extend(&item_id.to_le_bytes());
        encrypted_data.extend(&encrypted_quantity.to_le_bytes());
    }

    Ok(encrypted_data)
}
