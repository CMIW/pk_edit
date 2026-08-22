//! Bag inventory storage shared by BDSP and Luminescent Platinum saves.
//!
//! Unlike Gen III — where a pocket is a contiguous, XOR-encrypted list of
//! `(item_id, quantity)` slots — Gen 8b saves reserve one fixed 16-byte record
//! for *every* item ID in the game. The item ID is therefore implied by the
//! record's position rather than stored inside it:
//!
//! ```text
//! record_offset = ITEM_STORAGE_OFFSET + (RECORD_SIZE * item_id)
//! ```
//!
//! Each record is laid out as:
//!
//! | Offset | Size | Field                                   |
//! |--------|------|-----------------------------------------|
//! | `0x00` | 4    | quantity (`u32` LE)                     |
//! | `0x04` | 4    | "not new" flag (`0` = newly acquired)   |
//! | `0x08` | 4    | favourite flag (`1` = favourite)        |
//! | `0x0C` | 2    | sort order (`0` = never obtained)       |
//! | `0x0E` | 2    | padding                                 |
//!
//! An item counts as owned only when its sort order is non-zero; a zero sort
//! order means the player has never held it, regardless of the quantity bytes.
//! Nothing here is encrypted.

use std::collections::HashSet;

use byteorder::{ByteOrder, LittleEndian};
use rusqlite::Connection;

use crate::common::types::{BagItem, Pocket};
use crate::error::SaveDataError;

/// Base offset of the item inventory block. Identical in BDSP and Lumi.
pub const ITEM_STORAGE_OFFSET: usize = 0x0563C;

/// Size of a single item record.
pub const RECORD_SIZE: usize = 0x10;

/// Number of item records reserved by the save format.
pub const RECORD_COUNT: usize = 3000;

/// Largest quantity the games will display for a stackable item.
pub const MAX_QUANTITY: u16 = 999;

/// Byte offset of the record describing `item_id`.
const fn record_offset(item_id: u16) -> usize {
    ITEM_STORAGE_OFFSET + RECORD_SIZE * item_id as usize
}

/// Loads the `(item_id, name)` pairs a pocket may contain for `game_family`.
///
/// The item ID doubles as the record index, so this both bounds the search and
/// resolves names in one query.
///
/// # Errors
/// Returns [`SaveDataError::Unexpected`] if the items database is unreadable.
pub fn pocket_candidates(
    game_family: &str,
    pocket: Pocket,
) -> Result<Vec<(u16, String)>, SaveDataError> {
    let query = |db: &str| -> rusqlite::Result<Vec<(u16, String)>> {
        let conn = Connection::open(db)?;
        let mut stmt = conn.prepare(
            "SELECT id_in_game, name_en FROM items \
             WHERE game_family = ?1 AND pocket = ?2 ORDER BY id_in_game",
        )?;
        let rows = stmt.query_map([game_family, pocket.as_db_str()], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    };

    query("pk_edit.db").map_err(|e| SaveDataError::Unexpected(e.to_string()))
}

/// A single decoded inventory record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemRecord {
    pub quantity: u16,
    pub is_new: bool,
    pub is_favorite: bool,
    pub sort_order: u16,
}

impl ItemRecord {
    /// Returns `true` if the player has ever obtained this item.
    ///
    /// Mirrors PKHeX's `IsValidSaveSortNumberCount`.
    pub const fn is_obtained(&self) -> bool {
        self.sort_order != 0
    }
}

/// Reads the record for `item_id`.
///
/// # Errors
/// Returns [`SaveDataError::InvalidOffset`] if the record lies outside `data`.
pub fn read_record(data: &[u8], item_id: u16) -> Result<ItemRecord, SaveDataError> {
    let offset = record_offset(item_id);
    let bytes = data
        .get(offset..offset + RECORD_SIZE)
        .ok_or(SaveDataError::InvalidOffset(offset))?;

    // Quantities are stored as u32 but the games cap far below u16::MAX.
    let quantity = u16::try_from(LittleEndian::read_u32(bytes)).unwrap_or(u16::MAX);
    let is_new = LittleEndian::read_u32(&bytes[4..]) == 0;
    let is_favorite = LittleEndian::read_u32(&bytes[8..]) == 1;
    let sort_order = LittleEndian::read_u16(&bytes[12..]);

    Ok(ItemRecord {
        quantity,
        is_new,
        is_favorite,
        sort_order,
    })
}

/// Writes `record` into the slot for `item_id`.
///
/// # Errors
/// Returns [`SaveDataError::InvalidOffset`] if the record lies outside `data`.
pub fn write_record(
    data: &mut [u8],
    item_id: u16,
    record: ItemRecord,
) -> Result<(), SaveDataError> {
    let offset = record_offset(item_id);
    let bytes = data
        .get_mut(offset..offset + RECORD_SIZE)
        .ok_or(SaveDataError::InvalidOffset(offset))?;

    LittleEndian::write_u32(bytes, u32::from(record.quantity));
    LittleEndian::write_u32(&mut bytes[4..], u32::from(!record.is_new));
    LittleEndian::write_u32(&mut bytes[8..], u32::from(record.is_favorite));
    LittleEndian::write_u16(&mut bytes[12..], record.sort_order);
    LittleEndian::write_u16(&mut bytes[14..], 0);

    Ok(())
}

/// Clears the record for `item_id`, marking it as never obtained.
///
/// # Errors
/// Returns [`SaveDataError::InvalidOffset`] if the record lies outside `data`.
pub fn clear_record(data: &mut [u8], item_id: u16) -> Result<(), SaveDataError> {
    let offset = record_offset(item_id);
    let bytes = data
        .get_mut(offset..offset + RECORD_SIZE)
        .ok_or(SaveDataError::InvalidOffset(offset))?;
    bytes.fill(0);
    Ok(())
}

/// Returns the highest sort order currently used by any item in `candidates`.
fn highest_sort_order(data: &[u8], candidates: &[(u16, String)]) -> u16 {
    candidates
        .iter()
        .filter_map(|&(id, _)| read_record(data, id).ok())
        .map(|record| record.sort_order)
        .max()
        .unwrap_or(0)
}

/// Reads every obtained item belonging to `pocket`.
///
/// `candidates` lists the `(item_id, name)` pairs the pocket may contain, which
/// the caller sources from the items database. Results are ordered by the
/// in-game sort order so the list matches what the player sees in the bag.
///
/// # Errors
/// Returns an error if a candidate's record lies outside `data`.
pub fn read_pocket(
    data: &[u8],
    candidates: &[(u16, String)],
) -> Result<Vec<BagItem>, SaveDataError> {
    let mut owned: Vec<(u16, BagItem)> = Vec::new();

    for (id, name) in candidates {
        let record = read_record(data, *id)?;
        if record.is_obtained() {
            owned.push((
                record.sort_order,
                BagItem {
                    id: *id,
                    name: name.clone(),
                    quantity: record.quantity,
                },
            ));
        }
    }

    owned.sort_by_key(|&(sort_order, _)| sort_order);

    Ok(owned.into_iter().map(|(_, item)| item).collect())
}

/// Writes `items` as the complete contents of `pocket`.
///
/// Entries are identified by [`BagItem::id`]; the display name is ignored. This
/// matters because several IDs can share a name, so resolving by name would
/// redirect writes onto the wrong record and clear the original.
///
/// Any candidate absent from `items` is cleared, so removing an entry from the
/// list removes it from the bag. Key Items ignore `quantity` and are always
/// stored with a count of 1.
///
/// # Errors
/// Returns an error if a record lies outside `data`.
pub fn write_pocket(
    data: &mut [u8],
    pocket: Pocket,
    candidates: &[(u16, String)],
    items: &[BagItem],
) -> Result<(), SaveDataError> {
    let valid: HashSet<u16> = candidates.iter().map(|&(id, _)| id).collect();
    let base_sort = highest_sort_order(data, candidates);
    let mut written: HashSet<u16> = HashSet::with_capacity(items.len());

    for (index, item) in items.iter().enumerate() {
        // IDs not valid for this pocket are silently skipped, matching PKHeX's
        // behaviour of ignoring illegal slots.
        if !valid.contains(&item.id) || !written.insert(item.id) {
            continue;
        }

        let quantity = if pocket.has_quantity() {
            item.quantity.min(MAX_QUANTITY)
        } else {
            1
        };
        if quantity == 0 {
            written.remove(&item.id);
            continue;
        }

        let existing = read_record(data, item.id)?;
        // Preserve the slot's place in the bag if it was already obtained,
        // otherwise append it after everything currently held.
        let sort_order = if existing.is_obtained() {
            existing.sort_order
        } else {
            base_sort
                .saturating_add(u16::try_from(index).unwrap_or(u16::MAX))
                .saturating_add(1)
        };

        write_record(
            data,
            item.id,
            ItemRecord {
                quantity,
                is_new: existing.is_new && !existing.is_obtained(),
                is_favorite: existing.is_favorite,
                sort_order,
            },
        )?;
    }

    for &(id, _) in candidates {
        if !written.contains(&id) {
            clear_record(data, id)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buffer() -> Vec<u8> {
        vec![0u8; ITEM_STORAGE_OFFSET + RECORD_SIZE * RECORD_COUNT]
    }

    fn candidates() -> Vec<(u16, String)> {
        vec![
            (1, "Master Ball".to_string()),
            (2, "Ultra Ball".to_string()),
            (3, "Great Ball".to_string()),
        ]
    }

    fn item(id: u16, name: &str, quantity: u16) -> BagItem {
        BagItem {
            id,
            name: name.to_string(),
            quantity,
        }
    }

    #[test]
    fn record_roundtrip() {
        let mut data = buffer();
        let record = ItemRecord {
            quantity: 42,
            is_new: false,
            is_favorite: true,
            sort_order: 7,
        };
        write_record(&mut data, 3, record).expect("write");
        assert_eq!(read_record(&data, 3).expect("read"), record);
    }

    #[test]
    fn zero_sort_order_means_not_obtained() {
        let mut data = buffer();
        write_record(
            &mut data,
            5,
            ItemRecord {
                quantity: 10,
                is_new: true,
                is_favorite: false,
                sort_order: 0,
            },
        )
        .expect("write");
        assert!(!read_record(&data, 5).expect("read").is_obtained());
    }

    #[test]
    fn pocket_roundtrip_preserves_order() {
        let mut data = buffer();
        let candidates = candidates();
        let items = vec![item(2, "Ultra Ball", 20), item(1, "Master Ball", 1)];

        write_pocket(&mut data, Pocket::Balls, &candidates, &items).expect("write");
        let read = read_pocket(&data, &candidates).expect("read");

        assert_eq!(read, items);
    }

    #[test]
    fn omitted_items_are_cleared() {
        let mut data = buffer();
        let candidates = candidates();

        let all = vec![item(1, "Master Ball", 1), item(2, "Ultra Ball", 5)];
        write_pocket(&mut data, Pocket::Balls, &candidates, &all).expect("write all");

        let fewer = vec![item(1, "Master Ball", 1)];
        write_pocket(&mut data, Pocket::Balls, &candidates, &fewer).expect("write fewer");

        assert_eq!(read_pocket(&data, &candidates).expect("read"), fewer);
    }

    #[test]
    fn quantity_is_clamped() {
        let mut data = buffer();
        let candidates = candidates();
        let items = vec![item(2, "Ultra Ball", 5000)];

        write_pocket(&mut data, Pocket::Balls, &candidates, &items).expect("write");

        let read = read_pocket(&data, &candidates).expect("read");
        assert_eq!(read.first().map(|i| i.quantity), Some(MAX_QUANTITY));
    }

    #[test]
    fn key_items_always_store_one() {
        let mut data = buffer();
        let candidates = vec![(428, "Explorer Kit".to_string())];
        let items = vec![item(428, "Explorer Kit", 0)];

        write_pocket(&mut data, Pocket::Key, &candidates, &items).expect("write");

        let read = read_pocket(&data, &candidates).expect("read");
        assert_eq!(read, vec![item(428, "Explorer Kit", 1)]);
    }

    /// Luminescent lists several distinct ball IDs under the same display name
    /// (e.g. "Poké Ball" at 4, 1622, 1640 and 1710). Resolving entries by name
    /// used to redirect every write onto the first matching ID, which cleared
    /// the record the item actually lived in and dropped unrelated items from
    /// the pocket.
    #[test]
    fn duplicate_names_do_not_clobber_other_items() {
        let candidates = vec![
            (4, "Poké Ball".to_string()),
            (12, "Premier Ball".to_string()),
            (1622, "Poké Ball".to_string()),
            (1640, "Poké Ball".to_string()),
            (1710, "Poké Ball".to_string()),
        ];

        let mut data = buffer();
        // The player's Poké Balls live in the *duplicate* record, not the first.
        let held = vec![item(1640, "Poké Ball", 8), item(12, "Premier Ball", 1)];
        write_pocket(&mut data, Pocket::Balls, &candidates, &held).expect("initial write");
        assert_eq!(read_pocket(&data, &candidates).expect("read"), held);

        // Editing the Poké Ball stack must not disturb the Premier Ball.
        let edited = vec![item(1640, "Poké Ball", 9), item(12, "Premier Ball", 1)];
        write_pocket(&mut data, Pocket::Balls, &candidates, &edited).expect("edit write");

        let read = read_pocket(&data, &candidates).expect("read");
        assert_eq!(read, edited);
        assert!(
            read.iter().any(|i| i.id == 12),
            "Premier Ball was dropped when editing a duplicate-named item"
        );
        // The stack must stay in its original record rather than migrating.
        assert_eq!(read_record(&data, 1640).expect("read").quantity, 9);
        assert!(!read_record(&data, 4).expect("read").is_obtained());
    }

    /// The same ID appearing twice in one write should collapse, not duplicate.
    #[test]
    fn repeated_ids_are_written_once() {
        let mut data = buffer();
        let candidates = candidates();
        let items = vec![item(2, "Ultra Ball", 5), item(2, "Ultra Ball", 9)];

        write_pocket(&mut data, Pocket::Balls, &candidates, &items).expect("write");

        let read = read_pocket(&data, &candidates).expect("read");
        assert_eq!(read, vec![item(2, "Ultra Ball", 5)]);
    }
}
