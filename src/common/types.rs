//!
//!
//!
//!

/// A high-level representation of the Trainer's metadata.
/// This aggregates data that is physically split between Section 0 (Info) and Section 1 (Money) in the save file.
#[derive(Debug, Clone)]
pub struct Trainer {
    pub name: String,
    pub gender: Gender,
    pub id: TrainerID,
    pub time_played: TimePlayed,
    pub money: u32,
}

/// Represents the player's internal Trainer ID.
///
/// The Trainer ID is split into two components:
/// - The **public ID** (lower 16 bits), which is visible in-game.
/// - The **private ID** (upper 16 bits), which is used internally for certain mechanics (e.g., shiny Pokémon).
#[derive(Debug, Copy, Clone, Default)]
pub struct TrainerID {
    pub public: u16,
    pub private: u16,
}

/// The time played, parsed from the 5-byte field at offset 0x0E in Section 0.
#[derive(Debug, Clone, Copy, Default)]
pub struct TimePlayed {
    /// Total hours played (may exceed 999).
    pub hours: u16,
    /// Minutes component (0–59).
    pub minutes: u8,
    /// Seconds component (0–59).
    pub seconds: u8,
    /// Frame counter within the current second (0–59 at 60 fps).
    pub frames: u8,
}

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
    /// Medicine items (healing potions, etc.) (Gen IV+).
    Medicine,
    /// Pokéballs.
    Balls,
    /// TMs/HMs.
    TMs,
    /// Berries.
    Berries,
    /// Mail items.
    Mail,
    /// Battle items (Gen IV+).
    Battle,
    /// Key Items (cannot be discarded).
    Key,
}

/// A Pokémon's gender as derived from `PID % 256` and the species gender ratio.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    /// Male.
    M,
    /// Female.
    F,
    /// Genderless (gender ratio = 255) or empty slot.
    #[default]
    None,
}

/// Pokérus infection status derived from the Pokérus byte in [`MiscBlock`].
#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum Pokerus {
    /// Pokérus strain = 0: never infected.
    #[default]
    None,
    /// Strain > 0 and days remaining > 0: currently contagious.
    Infected,
    /// Strain > 0 and days remaining = 0: immunity gained.
    Cured,
}

/// Evolution chain data deserialized from the `evolution` JSON column in the Pokédex table.
///
/// Each entry is `[method, value]`, e.g. `["Level", "16"]` or `["Item", "Fire Stone"]`.
#[derive(Debug, Clone)]
pub struct Evolution {
    pub evolves_into: Option<u16>,
    pub method: Option<String>,
    pub condition: Option<String>,
}

/// Computed in-battle stat values at the Pokémon's current level.
#[derive(Debug, Default, Copy, Clone)]
pub struct ComputedStats {
    pub hp: u16,
    pub attack: u16,
    pub defense: u16,
    pub sp_attack: u16,
    pub sp_defense: u16,
    pub speed: u16,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct StatBlock {
    pub hp: u8,
    pub attack: u8,
    pub defense: u8,
    pub special_attack: u8,
    pub special_defense: u8,
    pub speed: u8,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct HyperTraining {
    pub hp: bool,
    pub attack: bool,
    pub defense: bool,
    pub special_attack: bool,
    pub special_defense: bool,
    pub speed: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Move {
    pub name: String,
    pub move_type: String,
    pub pp: u8,
    pub pp_used: u8,
}

#[derive(Debug, Clone)]
pub struct Abilities {
    pub slot1: String,
    pub slot2: Option<String>,
    pub hidden: Option<String>,
}
