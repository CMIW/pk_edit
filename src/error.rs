//! Error types for the `pk_edit` library.
//!
//! Three error enumerations are provided:
//! - [`DetectError`] for failures when detecting a save file's generation.
//! - [`PokemonError`] for failures when parsing or mutating a Pokémon.
//! - [`SaveDataError`] for failures when reading or writing a save file.

use std::ops::Range;
use thiserror::Error;

/// Errors that can occur when detecting a save file's generation from raw bytes.
#[derive(Error, Debug)]
pub enum DetectError {
    /// The buffer is smaller than the minimum supported save size.
    #[error("File too small: {0} bytes")]
    TooSmall(usize),
    /// No supported format matched the file size or magic bytes.
    #[error("Unknown save format (size {0} bytes)")]
    UnknownFormat(usize),
    /// A section checksum in the save file is invalid.
    #[error("Checksum validation failed: {0}")]
    ChecksumFailed(#[from] SaveDataError),
}

/// Errors that can occur while parsing or mutating a Pokémon.
#[derive(Error, Debug)]
pub enum PokemonError {
    /// The byte slice passed to `Pokemon::from_bytes` was shorter than 80 bytes.
    #[error("Invalid data length: expected at least 80 bytes, found {0}")]
    InvalidDataLength(usize),
    /// The species name does not match any entry in the Pokédex database.
    #[error("Species '{0}' not recognized")]
    UnknownSpecies(String),
    /// The item name does not match any entry in the items database.
    #[error("Item '{0}' not recognized")]
    UnknownItem(String),
    /// The move name does not match any entry in the moves database.
    #[error("Move '{0}' not recognized")]
    UnknownMove(String),
    /// The move slot index was out of the valid range (0–3).
    #[error("Invalid move slot {0} (must be 0-3)")]
    InvalidMoveSlot(usize),
    /// The Pokédex number had no associated gender ratio in the database.
    #[error("Gender ratio data missing for dex number {0}")]
    MissingGenderRatio(u16),
    /// The Pokéball ID was outside the valid range (1–12).
    #[error("Invalid pokeball {0} (must be 1-12)")]
    InvalidPokeball(u8),
    /// Invalid index or out-of-bounds access
    #[error("Invalid index: {0}")]
    InvalidIndex(usize),
}

/// Represents errors that can occur while handling save data.
#[derive(Error, Debug)]
pub enum SaveDataError {
    /// Section not found by ID
    #[error("Section not found for ID {0:?}")]
    SectionNotFound(String),

    /// Invalid data length encountered
    #[error("Invalid data length: expected {expected}, found {found}")]
    InvalidDataLength { expected: usize, found: usize },

    /// Invalid offset or out-of-bounds access
    #[error("Invalid offset: {0}")]
    InvalidOffset(usize),

    /// Invalid index or out-of-bounds access
    #[error("Invalid index: {0}")]
    InvalidIndex(usize),

    /// A byte range access fell outside the valid section data bounds.
    #[error("Invalid range: {0:?}")]
    InvalidRange(Range<usize>),

    /// Decryption failure
    #[error("Decryption failed for key {0:#X}")]
    DecryptionError(u16),

    /// Checksum mismatch detected
    #[error("Checksum mismatch: expected {expected:#X}, found {found:#X}")]
    ChecksumMismatch { expected: u16, found: u16 },

    /// Unexpected error occurred
    #[error("Unexpected error: {0}")]
    Unexpected(String),

    /// A [`PokemonError`] bubbled up while processing party or PC data.
    #[error("Pokemon parsing Error:{0}")]
    PokemonError(#[from] PokemonError),

    #[error("Party is full (max 6)")]
    PartyFull,

    #[error("Cannot remove the last Pokémon from party")]
    CannotEmptyParty,

    #[error("Invalid party slot {0}")]
    InvalidPartySlot(usize),
}
