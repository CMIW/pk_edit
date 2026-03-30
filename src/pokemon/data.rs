//! Raw data sub-structures that make up the encrypted portion of a Gen III Pokémon.
//!
//! Each [`crate::pokemon::Pokemon`] contains 48 bytes of XOR-encrypted data split into four
//! 12-byte blocks whose order is determined by `PID % 24` (see [`BLOCK_ORDERS`]):
//!
//! | Block | Contents |
//! |---|---|
//! | [`GrowthBlock`] | Species, held item, experience, PP bonuses, friendship |
//! | [`AttacksBlock`] | Up to 4 move IDs and their current PP |
//! | [`EVsBlock`] | Effort values for all 6 stats + contest stats |
//! | [`MiscBlock`] | Pokérus, met location, origins, IVs, ribbons |

use modular_bitfield::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

/// The Growth sub-structure (12 bytes): species, held item, experience, PP bonuses, friendship.
#[derive(Default, Copy, Clone, Debug)]
pub struct GrowthBlock {
    /// Internal Gen III species ID (not National Dex number for species ≥ 252).
    pub species: u16,
    /// Gen III item ID of the held item (`0` = nothing).
    pub item: u16,
    /// Total accumulated experience points.
    pub experience: u32,
    /// PP-Up bonus bits packed per move slot.
    pub pp_bonuses: u8,
    /// Friendship / happiness value (0–255).
    pub friendship: u8,
    /// Unused padding.
    pub unknown: u16,
}

/// The Attacks sub-structure (12 bytes): up to four move IDs and their current PP.
#[derive(Default, Copy, Clone, Debug)]
pub struct AttacksBlock {
    /// Move IDs for slots 0–3 (`0` = empty slot).
    pub moves: [u16; 4],
    /// Current PP remaining for each move slot.
    pub pp: [u8; 4],
}

/// The EVs sub-structure (12 bytes): effort values for battle stats and contest stats.
#[derive(Default, Copy, Clone, Debug)]
pub struct EVsBlock {
    /// HP effort value (0–255, total across all 6 stats ≤ 510).
    pub hp: u8,
    /// Attack EV.
    pub attack: u8,
    /// Defense EV.
    pub defense: u8,
    /// Speed EV.
    pub speed: u8,
    /// Special Attack EV.
    pub sp_attack: u8,
    /// Special Defense EV.
    pub sp_defense: u8,
    /// Contest coolness stat.
    pub coolness: u8,
    /// Contest beauty stat.
    pub beauty: u8,
    /// Contest cuteness stat.
    pub cuteness: u8,
    /// Contest smartness stat.
    pub smartness: u8,
    /// Contest toughness stat.
    pub toughness: u8,
    /// Pokéblock feel stat.
    pub feel: u8,
}

#[bitfield]
#[derive(Default, Copy, Clone, Debug)]
pub struct OriginsInfo {
    pub level_met: B7,
    pub game_of_origin: B4,
    pub pokeball: B4,
    pub ot_gender: B1,
}

#[bitfield]
#[derive(Default, Copy, Clone, Debug)]
pub struct IVsEggAbility {
    pub hp_iv: B5,
    pub attack_iv: B5,
    pub defense_iv: B5,
    pub speed_iv: B5,
    pub sp_attack_iv: B5,
    pub sp_defense_iv: B5,
    pub is_egg: B1,
    pub ability_bit: B1,
}

#[bitfield]
#[derive(Default, Copy, Clone, Debug)]
pub struct RibbonsObedience {
    pub cool_ribbons: B3,
    pub beauty_ribbons: B3,
    pub cute_ribbons: B3,
    pub smart_ribbons: B3,
    pub tough_ribbons: B3,
    pub champion_ribbon: B1,
    pub winning_ribbon: B1,
    pub victory_ribbon: B1,
    pub artist_ribbon: B1,
    pub effort_ribbon: B1,
    pub marine_ribbon: B1,
    pub land_ribbon: B1,
    pub sky_ribbon: B1,
    pub country_ribbon: B1,
    pub national_ribbon: B1,
    pub earth_ribbon: B1,
    pub world_ribbon: B1,
    pub padding: B4,
    pub obedience: B1,
}

/// The Misc sub-structure (12 bytes): Pokérus, met location, origins, IVs, and ribbons.
#[derive(Default, Copy, Clone, Debug)]
pub struct MiscBlock {
    /// Pokérus byte: upper nibble = strain (1–15), lower nibble = days remaining (0 = cured).
    pub pokerus: u8,
    /// Map location ID where the Pokémon was met.
    pub met_location: u8,
    /// Packed bitfield: level met, game of origin, Pokéball, OT gender.
    pub origins_info: OriginsInfo,
    /// Packed bitfield: six IVs (5 bits each), is-egg flag, ability bit.
    pub iv_egg_ability: IVsEggAbility,
    /// Packed bitfield: contest ribbons, battle ribbons, obedience flag.
    pub ribbons_obedience: RibbonsObedience,
}

#[derive(Default, Copy, Clone, Debug)]
pub(crate) struct PokemonData {
    pub growth: GrowthBlock,
    pub attacks: AttacksBlock,
    pub evs: EVsBlock,
    pub misc: MiscBlock,
}

#[rustfmt::skip]
#[derive(Copy, Clone, Debug)]
pub(crate) enum BlockType { Growth, Attacks, EVs, Misc }

#[rustfmt::skip]
/// The 24 permutations of the data blocks (PID % 24).
pub const BLOCK_ORDERS: [[BlockType; 4]; 24] = [
    [BlockType::Growth, BlockType::Attacks, BlockType::EVs, BlockType::Misc], // 0
    [BlockType::Growth, BlockType::Attacks, BlockType::Misc, BlockType::EVs], // 1
    [BlockType::Growth, BlockType::EVs, BlockType::Attacks, BlockType::Misc], // 2
    [BlockType::Growth, BlockType::EVs, BlockType::Misc, BlockType::Attacks], // 3
    [BlockType::Growth, BlockType::Misc, BlockType::Attacks, BlockType::EVs], // 4
    [BlockType::Growth, BlockType::Misc, BlockType::EVs, BlockType::Attacks], // 5
    [BlockType::Attacks, BlockType::Growth, BlockType::EVs, BlockType::Misc], // 6
    [BlockType::Attacks, BlockType::Growth, BlockType::Misc, BlockType::EVs], // 7
    [BlockType::Attacks, BlockType::EVs, BlockType::Growth, BlockType::Misc], // 8
    [BlockType::Attacks, BlockType::EVs, BlockType::Misc, BlockType::Growth], // 9
    [BlockType::Attacks, BlockType::Misc, BlockType::Growth, BlockType::EVs], // 10
    [BlockType::Attacks, BlockType::Misc, BlockType::EVs, BlockType::Growth], // 11
    [BlockType::EVs, BlockType::Growth, BlockType::Attacks, BlockType::Misc], // 12
    [BlockType::EVs, BlockType::Growth, BlockType::Misc, BlockType::Attacks], // 13
    [BlockType::EVs, BlockType::Attacks, BlockType::Growth, BlockType::Misc], // 14
    [BlockType::EVs, BlockType::Attacks, BlockType::Misc, BlockType::Growth], // 15
    [BlockType::EVs, BlockType::Misc, BlockType::Growth, BlockType::Attacks], // 16
    [BlockType::EVs, BlockType::Misc, BlockType::Attacks, BlockType::Growth], // 17
    [BlockType::Misc, BlockType::Growth, BlockType::Attacks, BlockType::EVs], // 18
    [BlockType::Misc, BlockType::Growth, BlockType::EVs, BlockType::Attacks], // 19
    [BlockType::Misc, BlockType::Attacks, BlockType::Growth, BlockType::EVs], // 20
    [BlockType::Misc, BlockType::Attacks, BlockType::EVs, BlockType::Growth], // 21
    [BlockType::Misc, BlockType::EVs, BlockType::Growth, BlockType::Attacks], // 22
    [BlockType::Misc, BlockType::EVs, BlockType::Attacks, BlockType::Growth], // 23
];

/// The language code stored in a Pokémon's header byte (offset 0x12).
#[derive(Debug)]
pub enum Language {
    /// Language code 1.
    Japanese,
    /// Language code 2.
    English,
    /// Language code 3.
    French,
    /// Language code 4.
    Italian,
    /// Language code 5.
    German,
    /// Any code not in 1–7 (treated as no language).
    Unused,
    /// Language code 7.
    Spanish,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Unused => write!(f, ""),
            Language::French => write!(f, "FRE"),
            Language::German => write!(f, "GER"),
            Language::English => write!(f, "ENG"),
            Language::Italian => write!(f, "ITA"),
            Language::Spanish => write!(f, "SPA"),
            Language::Japanese => write!(f, "JAP"),
        }
    }
}

impl From<u8> for Language {
    fn from(language: u8) -> Self {
        match language {
            1 => Language::Japanese,
            2 => Language::English,
            3 => Language::French,
            4 => Language::Italian,
            5 => Language::German,
            7 => Language::Spanish,
            _ => Language::Unused,
        }
    }
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Evolution {
    /// The pre-evolution entry: `[method, value]`, or `None` if this is a base form.
    prev: Option<[String; 2]>,
    /// One or more evolution targets, or `None` if this is a final form.
    next: Option<Vec<[String; 2]>>,
}

impl Evolution {
    pub fn prev_level(&mut self) -> Option<u8> {
        if let Some(prev) = &self.prev {
            let level_str = &prev[1].replace("Level ", "");

            level_str.parse::<u8>().ok()
        } else {
            None
        }
    }
}
