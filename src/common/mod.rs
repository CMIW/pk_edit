//! Common utilities shared across the `pk_edit` library.
//!
//! - [`charset`] provides bidirectional conversion between the Gen III custom
//!   256-character encoding and Unicode.
//! - [`inventory8b`] implements the bag storage format shared by BDSP and Lumi.
//! - [`types`] holds cross-generation data types.

pub mod charset;
pub mod inventory8b;
pub mod types;

/// Looks up the English flavor text for an item in the given game family.
///
/// Returns an empty string if the database is unreadable, the item is unknown,
/// or the item has no recorded description.
pub fn item_flavor_text_for(game_family: &str, name: &str) -> String {
    let Ok(conn) = rusqlite::Connection::open("pk_edit.db") else {
        return String::new();
    };
    let res = conn.query_row(
        "SELECT flavor_text FROM items WHERE name_en = ?1 AND game_family = ?2",
        [name, game_family],
        |row| row.get::<_, Option<String>>(0),
    );
    let _ = conn.close();
    res.unwrap_or(None).unwrap_or_default()
}
