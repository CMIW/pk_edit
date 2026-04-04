//! Trainer metadata structures parsed from Section 0 (Trainer Info) and Section 1 (Team/Items).
//!
//! The primary type is [`Trainer`], which aggregates data physically split across two save
//! sections. Supporting types include [`TrainerID`], [`TimePlayed`], [`GameVersion`], and
//! [`GymBadges`].

use byteorder::{ByteOrder, LittleEndian};
use modular_bitfield::prelude::*;
use std::fmt;

use crate::common::types::{Gender, TrainerID};

/// The game version identified by the game code stored in Section 0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameVersion {
    /// Game code `0x00000000`.
    RubySapphire,
    /// Game code `0x00000001`.
    FireRedLeafGreen,
    /// Any other game code.
    Emerald,
}

impl From<[u8; 4]> for TrainerID {
    fn from(buffer: [u8; 4]) -> Self {
        TrainerID {
            public: LittleEndian::read_u16(&buffer[..2]),
            private: LittleEndian::read_u16(&buffer[2..]),
        }
    }
}

impl From<TrainerID> for Vec<u8> {
    fn from(val: TrainerID) -> Self {
        let mut buf = [0u8; 4];
        LittleEndian::write_u16(&mut buf[..2], val.public);
        LittleEndian::write_u16(&mut buf[2..], val.private);
        buf.to_vec()
    }
}

impl fmt::Display for TrainerID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:05}", self.public)
    }
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

impl TimePlayed {
    // Helper to parse from the 5-byte chunk found in Section 0
    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            hours: LittleEndian::read_u16(&data[0..2]),
            minutes: data[2],
            seconds: data[3],
            frames: data[4],
        }
    }

    pub fn to_bytes(&self) -> [u8; 5] {
        let mut buf = [0u8; 5];
        LittleEndian::write_u16(&mut buf[0..2], self.hours);
        buf[2] = self.minutes;
        buf[3] = self.seconds;
        buf[4] = self.frames;
        buf
    }
}

#[bitfield]
#[derive(Debug, Clone, Copy, Default)]
pub struct GymBadges {
    pub badge_1: B1, // Stone (RS/E) / Boulder (FRLG)
    pub badge_2: B1, // Knuckle (RS/E) / Cascade (FRLG)
    pub badge_3: B1, // Dynamo (RS/E) / Thunder (FRLG)
    pub badge_4: B1, // Heat (RS/E) / Rainbow (FRLG)
    pub badge_5: B1, // Balance (RS/E) / Soul (FRLG)
    pub badge_6: B1, // Feather (RS/E) / Marsh (FRLG)
    pub badge_7: B1, // Mind (RS/E) / Volcano (FRLG)
    pub badge_8: B1, // Rain (RS/E) / Earth (FRLG)
}

/// A high-level representation of the Trainer's metadata.
/// This aggregates data that is physically split between Section 0 (Info) and Section 1 (Money) in the save file.
#[derive(Debug, Clone)]
pub struct Trainer {
    pub name: String,
    pub gender: Gender,
    pub id: TrainerID,
    pub time_played: TimePlayed,
    pub money: u32,
    pub game_version: GameVersion,
    pub security_key: u32, // Kept for debugging/reference
}
