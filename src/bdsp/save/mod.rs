pub mod pc;
pub mod trainer;

use crate::bdsp::pokemon::BdspPokemon;
use crate::common::inventory8b;
use crate::common::types::{BagItem, Pocket, StorageType, Trainer};
use crate::error::{PokemonError, SaveDataError};
use crate::traits::pokemon::Pokemon as _;
use byteorder::{ByteOrder, LittleEndian};
use md5::{Md5, Digest};

const PARTY_OFFSET: usize = 0x14098;
const PARTY_COUNT_OFFSET: usize = PARTY_OFFSET + 6 * 0x158;
const PARTY_SLOT_SIZE: usize = 0x158;

/// `game_family` value used for BDSP rows in the items database.
const GAME_FAMILY: &str = "bdsp";

/// Fixed hash offset used by all BDSP versions (V1.0–V1.3).
const HASH_OFFSET: usize = 0xE9818;
const HASH_SIZE: usize = 0x10;

/// (file_size, expected_version_marker)
const VALID_SIZES: &[(usize, u32)] = &[
    (0xE9828, 0x25), // V1_0
    (0xEDC20, 0x2C), // V1_1
    (0xEED8C, 0x32), // V1_2
    (0xEF0A4, 0x34), // V1_3
];

#[derive(Debug)]
pub struct SaveFile {
    data: Vec<u8>,
}

impl SaveFile {
    pub fn new(data: &[u8]) -> Result<Self, SaveDataError> {
        let version = byteorder::LittleEndian::read_u32(data.get(0..4).unwrap_or(&[0u8; 4]));
        let is_valid = VALID_SIZES
            .iter()
            .any(|&(sz, ver)| data.len() == sz && version == ver);
        if !is_valid {
            return Err(SaveDataError::InvalidDataLength {
                expected: 0xEF0A4,
                found: data.len(),
            });
        }
        let save = Self { data: data.to_vec() };
        save.validate_hash()?;
        Ok(save)
    }

    pub fn party(&self) -> Result<Vec<BdspPokemon>, PokemonError> {
        let mut party = Vec::with_capacity(6);
        for i in 0..6 {
            let offset = PARTY_OFFSET + i * PARTY_SLOT_SIZE;
            if let Some(slot_data) = self.data.get(offset..offset + PARTY_SLOT_SIZE) {
                party.push(BdspPokemon::from_bytes(offset, slot_data)?);
            }
        }
        Ok(party)
    }

    pub fn pc_box(&self, number: usize) -> Result<Vec<BdspPokemon>, PokemonError> {
        pc::read_box(&self.data, number)
    }

    pub fn save_pokemon(
        &mut self,
        storage: StorageType,
        pokemon: BdspPokemon,
    ) -> Result<(), SaveDataError> {
        let encrypted = pokemon.to_bytes();
        let offset = pokemon.offset;
        match storage {
            StorageType::Party | StorageType::PC => {
                if let Some(dest) = self.data.get_mut(offset..offset + encrypted.len()) {
                    dest.copy_from_slice(&encrypted);
                }
            }
            StorageType::None => {}
        }
        self.update_party_count();
        Ok(())
    }

    pub fn swap_pokemon(
        &mut self,
        mut from: BdspPokemon,
        from_s: StorageType,
        mut to: BdspPokemon,
        to_s: StorageType,
    ) -> Result<(), SaveDataError> {
        std::mem::swap(&mut from.offset, &mut to.offset);
        self.save_pokemon(to_s, from)?;
        self.save_pokemon(from_s, to)?;
        Ok(())
    }

    fn update_party_count(&mut self) {
        let mut highest = 0usize;
        for i in 0..6 {
            let offset = PARTY_OFFSET + i * PARTY_SLOT_SIZE;
            if let Some(slot_data) = self.data.get(offset..offset + PARTY_SLOT_SIZE) {
                if let Ok(pk) = BdspPokemon::from_bytes(offset, slot_data) {
                    if !pk.is_empty() {
                        highest = i + 1;
                    }
                }
            }
        }
        if let Some(count) = self.data.get_mut(PARTY_COUNT_OFFSET) {
            *count = highest as u8;
        }
        self.update_hash();
    }

    /// Returns the items currently held in `pocket`.
    ///
    /// Only items the player has actually obtained are returned, ordered to
    /// match the in-game bag.
    ///
    /// # Errors
    /// Returns an error if the items database is unreadable or a record lies
    /// outside the save buffer.
    pub fn pocket(&self, pocket: Pocket) -> Result<Vec<BagItem>, SaveDataError> {
        let candidates = inventory8b::pocket_candidates(GAME_FAMILY, pocket)?;
        inventory8b::read_pocket(&self.data, &candidates)
    }

    /// Replaces the contents of `pocket` with `items`.
    ///
    /// Items omitted from `items` are removed from the bag.
    ///
    /// # Errors
    /// Returns an error if the items database is unreadable or a record lies
    /// outside the save buffer.
    pub fn save_pocket(
        &mut self,
        pocket: Pocket,
        items: Vec<BagItem>,
    ) -> Result<(), SaveDataError> {
        let candidates = inventory8b::pocket_candidates(GAME_FAMILY, pocket)?;
        inventory8b::write_pocket(&mut self.data, pocket, &candidates, &items)?;
        self.update_hash();
        Ok(())
    }

    pub fn trainer(&self) -> Result<Trainer, SaveDataError> {
        trainer::parse_trainer(&self.data)
    }

    pub fn save_trainer(&mut self, trainer: &Trainer) -> Result<(), SaveDataError> {
        trainer::write_trainer(&mut self.data, trainer)?;
        self.update_hash();
        Ok(())
    }

    pub fn raw_data(&self) -> Vec<u8> {
        self.data.clone()
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    pub fn is_pc_empty(&self) -> bool {
        false // TODO: could check first box
    }

    fn validate_hash(&self) -> Result<(), SaveDataError> {
        let expected: Vec<u8> = self
            .data
            .get(HASH_OFFSET..HASH_OFFSET + HASH_SIZE)
            .ok_or(SaveDataError::InvalidOffset(HASH_OFFSET))?
            .to_vec();
        let computed = self.compute_hash();
        if computed != expected {
            return Err(SaveDataError::Unexpected(
                "MD5 hash mismatch".to_string(),
            ));
        }
        Ok(())
    }

    /// Returns the number of gym badges earned (0–8).
    pub fn badge_count(&self) -> u8 {
        const BADGE_OFFSET: usize = 0x79BB4 + 0x29;
        *self.data.get(BADGE_OFFSET).unwrap_or(&0)
    }

    /// Returns `(caught, seen)` Pokédex counts.
    ///
    /// BDSP stores a u32 state per species: 0=None, 1=HeardOf, 2=Seen, 3=Captured.
    /// There are 493 species (National Dex Gen 1–4).
    pub fn pokedex_counts(&self) -> (u16, u16) {
        const POKEDEX_OFFSET: usize = 0x7A328;
        const COUNT_SPECIES: usize = 493;

        let mut caught = 0u16;
        let mut seen = 0u16;

        for i in 0..COUNT_SPECIES {
            let offset = POKEDEX_OFFSET + i * 4;
            let state = match self.data.get(offset..offset + 4) {
                Some(bytes) => LittleEndian::read_u32(bytes),
                None => continue,
            };
            match state {
                2 => seen += 1,        // Seen
                3 => { seen += 1; caught += 1; } // Captured
                _ => {}                 // None or HeardOf
            }
        }

        (caught, seen)
    }

    /// Compute the MD5 over the entire file with the 16-byte hash region zeroed,
    /// matching PKHeX's algorithm.
    fn compute_hash(&self) -> Vec<u8> {
        let mut buf = self.data.clone();
        if let Some(hash_region) = buf.get_mut(HASH_OFFSET..HASH_OFFSET + HASH_SIZE) {
            hash_region.fill(0);
        }
        Vec::from(Md5::digest(&buf).as_slice())
    }

    fn update_hash(&mut self) {
        let hash = self.compute_hash();
        if let Some(dest) = self
            .data
            .get_mut(HASH_OFFSET..HASH_OFFSET + HASH_SIZE)
        {
            dest.copy_from_slice(&hash);
        }
    }
}
