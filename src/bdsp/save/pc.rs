use crate::bdsp::pokemon::BdspPokemon;
use crate::error::{PokemonError, SaveDataError};
use crate::traits::pokemon::Pokemon;

const BOX_DATA_START: usize = 0x14EF4;
const BOX_COUNT: usize = 40;
const SLOTS_PER_BOX: usize = 30;
const SLOT_SIZE: usize = 0x158;
const BOX_LAYOUT_OFFSET: usize = 0x148AA;
const CURRENT_BOX_OFFSET: usize = BOX_LAYOUT_OFFSET + 0x61E;

pub fn read_box(save_data: &[u8], box_num: usize) -> Result<Vec<BdspPokemon>, PokemonError> {
    if box_num >= BOX_COUNT {
        return Err(PokemonError::InvalidIndex(box_num));
    }
    let start = slot_offset(box_num, 0);
    let end = start + SLOTS_PER_BOX * SLOT_SIZE;
    let box_data = save_data
        .get(start..end)
        .ok_or(PokemonError::InvalidIndex(box_num))?;
    let mut list = Vec::with_capacity(SLOTS_PER_BOX);
    for (slot, chunk) in box_data.chunks(SLOT_SIZE).enumerate() {
        let offset = slot_offset(box_num, slot);
        list.push(BdspPokemon::from_bytes(offset, chunk)?);
    }
    Ok(list)
}

pub fn write_pokemon(
    save_data: &mut [u8],
    box_num: usize,
    slot: usize,
    pokemon: &BdspPokemon,
) -> Result<(), SaveDataError> {
    if box_num >= BOX_COUNT || slot >= SLOTS_PER_BOX {
        return Err(SaveDataError::InvalidIndex(box_num * SLOTS_PER_BOX + slot));
    }
    let offset = slot_offset(box_num, slot);
    let bytes = pokemon.to_bytes();
    let dest = save_data
        .get_mut(offset..offset + bytes.len())
        .ok_or(SaveDataError::InvalidOffset(offset))?;
    dest.copy_from_slice(&bytes);
    Ok(())
}

pub fn slot_offset(box_num: usize, slot: usize) -> usize {
    BOX_DATA_START + (box_num * SLOTS_PER_BOX + slot) * SLOT_SIZE
}

pub fn current_box(save_data: &[u8]) -> usize {
    save_data
        .get(CURRENT_BOX_OFFSET)
        .copied()
        .map(usize::from)
        .unwrap_or(0)
}

pub fn box_count() -> usize {
    BOX_COUNT
}
