use super::section::{Section, SectionID};
use crate::error::{PokemonError, SaveDataError};
use crate::pokemon::Pokemon;

const PC_BUFFER_SECTION_SIZE: usize = 0xF80; // 3968 bytes (Buffers A-H)
const PC_BUFFER_I_SECTION_SIZE: usize = 0x7D0; // 2000 bytes (Buffer I)
const PC_BOX_COUNT: usize = 14;
const PC_SLOTS_PER_BOX: usize = 30;
const PC_POKEMON_SIZE: usize = 80; // PC only stores persistent data
const PC_BUFFER_OFFSET: usize = 0x0004; // 4-byte cursor/padding at start

/// Representation of the PC Buffer.
///
/// In Pokémon Generation III games, the PC data is too large to fit in a single 4KB section.
/// It is fragmented across 9 sections (Buffer A through Buffer I).
/// This struct stitches them together into a single contiguous `data` vector for easy manipulation.
#[derive(Default, Debug, Clone)]
pub struct PCBuffer {
    /// The 9 sections that collectively store the PC data.
    buffer: [Section; 9],
    /// The combined data extracted from the sections.
    data: Vec<u8>,
}

impl PCBuffer {
    /// Creates a new PCBuffer by combining data from the specified sections.
    /// The PCBuffer is constructed by extracting data from the sections that contain the PC data.
    /// Special handling is required for the last section (`PCbufferI`), which may have a different size.
    pub(crate) fn new(buffer: [Section; 9], data_buffer: &[u8]) -> Self {
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
    pub(crate) fn pc_box(&self, number: usize) -> Result<Vec<Pokemon>, PokemonError> {
        let mut boxes = self.data[0x0004..0x8344].chunks(2400);
        let pc = boxes.nth(number).expect("Expected value but found None");
        let mut list: Vec<Pokemon> = vec![];

        for (i, pokemon) in pc.chunks(80).enumerate() {
            // data_offset + pc box offset + slot offset
            let offset = 0x0004 + (number * 2400) + (i * 80);
            let pokemon = Pokemon::from_bytes(offset, pokemon)?;
            list.push(pokemon);
        }

        Ok(list)
    }

    /// Saves a Pokémon back into the PC Buffer and updates the relevant sections.
    pub(crate) fn save_pokemon(
        &mut self,
        pokemon: Pokemon,
        buffer: &mut [u8],
    ) -> Result<(), SaveDataError> {
        let offset = pokemon.offset;
        // Validate offset is within PC bounds
        if offset < PC_BUFFER_OFFSET || offset >= self.data.len() {
            return Err(SaveDataError::InvalidOffset(offset));
        }

        self.data[offset..offset + 80].copy_from_slice(&pokemon.to_bytes()[..80]);
        self.sync_and_checksum(buffer)
    }

    /// Calculates the bytes offset within the PC Buffer for a specific box and slot.
    fn slot_offset(box_idx: usize, slot_idx: usize) -> Result<usize, SaveDataError> {
        if box_idx >= PC_BOX_COUNT || slot_idx >= PC_SLOTS_PER_BOX {
            return Err(SaveDataError::InvalidOffset(box_idx * 100 + slot_idx));
        }
        Ok(PC_BUFFER_OFFSET
            + (box_idx * PC_SLOTS_PER_BOX * PC_POKEMON_SIZE)
            + (slot_idx * PC_POKEMON_SIZE))
    }

    /// Writes 80 byts to a specific location and triggers a save sync.
    pub(crate) fn write_raw_slot(
        &mut self,
        box_idx: usize,
        slot_idx: usize,
        data: &[u8],
        main_buffer: &mut [u8],
    ) -> Result<(), SaveDataError> {
        let offset = Self::slot_offset(box_idx, slot_idx)?;

        if data.len() != PC_POKEMON_SIZE {
            return Err(SaveDataError::InvalidDataLength {
                expected: PC_POKEMON_SIZE,
                found: data.len(),
            });
        }

        self.data[offset..offset + PC_POKEMON_SIZE].copy_from_slice(data);
        self.sync_and_checksum(main_buffer)?;
        Ok(())
    }

    /// Syncs the modified PCBuffer back to the main SaveFile chunks and updates checksums.
    /// (Extracted logic from the original save_pokemon to avoid duplication)
    fn sync_and_checksum(&self, main_buffer: &mut [u8]) -> Result<(), SaveDataError> {
        for (i, section) in self.data.chunks(PC_BUFFER_SECTION_SIZE).enumerate() {
            // Handle the split nature of the PC buffer sections
            let target_len = if i == 8 {
                PC_BUFFER_I_SECTION_SIZE
            } else {
                PC_BUFFER_SECTION_SIZE
            };
            let target_offset = self.buffer[i].offset();

            main_buffer[target_offset..target_offset + target_len]
                .copy_from_slice(&section[..target_len]);
            self.buffer[i].write_checksum(main_buffer)?;
        }
        Ok(())
    }

    /// Checks if the PC Buffer is empty.
    pub(crate) fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
