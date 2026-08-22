//! Storage location types and bag-pocket helpers.
//!
//! [`StorageType`] identifies where a [`crate::pokemon::Pokemon`] lives (party, PC, or unset).
//! [`Pocket`] identifies the five bag pockets. [`pocket_address`] maps a pocket to its
//! byte range within Section 1, accounting for game-version differences.
//!
//! Pocket data in save files is XOR-encrypted with the lower 16 bits of the security key.
//! [`decrypt_pocket`] and [`encrypt_pocket`] handle the conversion to/from `(name, quantity)` pairs.

use crate::gen3::save::section::SectionID;
use byteorder::{ByteOrder, LittleEndian};
use std::fmt;

use crate::common::types::BagItem;
use crate::error::SaveDataError;
use rusqlite::Connection;

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

impl From<Pocket> for crate::common::types::Pocket {
    fn from(pocket: Pocket) -> Self {
        match pocket {
            Pocket::Items => Self::Items,
            Pocket::Pokeballs => Self::Balls,
            Pocket::Berries => Self::Berries,
            Pocket::Tms => Self::TMs,
            Pocket::Key => Self::Key,
        }
    }
}

impl TryFrom<crate::common::types::Pocket> for Pocket {
    type Error = SaveDataError;

    /// Maps a cross-generation pocket onto its Gen III equivalent.
    ///
    /// # Errors
    /// Returns [`SaveDataError::Unexpected`] for pockets Gen III does not have
    /// (Medicine, Mail, Battle, and Treasure).
    fn try_from(pocket: crate::common::types::Pocket) -> Result<Self, Self::Error> {
        use crate::common::types::Pocket as Common;
        match pocket {
            Common::Items => Ok(Self::Items),
            Common::Balls => Ok(Self::Pokeballs),
            Common::Berries => Ok(Self::Berries),
            Common::TMs => Ok(Self::Tms),
            Common::Key => Ok(Self::Key),
            other => Err(SaveDataError::Unexpected(format!(
                "Generation III has no {other} pocket"
            ))),
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
///
/// Returns every slot in order, including unoccupied ones as [`BagItem::empty`].
///
/// # Errors
/// Returns an error if the data is too short or decryption fails.
pub fn decrypt_pocket(data: &[u8], security_key: u16) -> Result<Vec<BagItem>, SaveDataError> {
    let mut pocket = Vec::new();

    for chunk in data.chunks(4) {
        if chunk.len() < 4 {
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

        let quantity = encrypted_quantity ^ security_key;

        if item_id == 0 {
            pocket.push(BagItem::empty());
            continue;
        }

        let name = Connection::open("pk_edit.db")
            .ok()
            .and_then(|conn| {
                conn.query_row(
                    "SELECT name_en FROM items WHERE id_in_game = ?1 AND game_family = 'gen3'",
                    [item_id as usize],
                    |row| row.get::<_, String>(0),
                )
                .ok()
            })
            .unwrap_or_else(|| format!("Unknown Item ({item_id})"));

        pocket.push(BagItem {
            id: item_id,
            name,
            quantity,
        });
    }

    Ok(pocket)
}

/// Encrypts pocket data using the security key for saving.
///
/// Returns raw bytes ready to be written to Section 1. Empty slots are written
/// as `item_id = 0` with a quantity of zero.
///
/// Entries carrying an `id` of zero but a real name — which the UI produces when
/// the user picks an item from a dropdown — are resolved by name as a fallback.
///
/// # Errors
/// This function does not fail, but returns `Result` for compatibility.
pub fn encrypt_pocket(pocket: Vec<BagItem>, security_key: u16) -> Result<Vec<u8>, SaveDataError> {
    let mut encrypted_data = Vec::new();

    for entry in pocket {
        if entry.is_empty() {
            encrypted_data.extend(&0u16.to_le_bytes());
            encrypted_data.extend(&0u16.to_le_bytes());
            continue;
        }

        let item_id = if entry.id != 0 {
            entry.id
        } else {
            Connection::open("pk_edit.db")
                .ok()
                .and_then(|conn| {
                    conn.query_row(
                        "SELECT id_in_game FROM items WHERE name_en = ?1 AND game_family = 'gen3'",
                        [entry.name.as_str()],
                        |row| row.get::<_, u16>(0),
                    )
                    .ok()
                })
                .unwrap_or(0)
        };

        let encrypted_quantity = entry.quantity ^ security_key;

        encrypted_data.extend(&item_id.to_le_bytes());
        encrypted_data.extend(&encrypted_quantity.to_le_bytes());
    }

    Ok(encrypted_data)
}
