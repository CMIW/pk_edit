use crate::error::PokemonError;
use crate::error::SaveDataError;

pub trait PCAccess {
    type Pokemon;
    fn pc_box(&self, box_idx: usize) -> Result<Vec<Self::Pokemon>, PokemonError>;
    fn delete_pokemon(&mut self, box_idx: usize, slot_idx: usize) -> Result<(), SaveDataError>;
    fn save_pokemon(
        &mut self,
        box_idx: usize,
        slot_idx: usize,
        pokemon: &Self::Pokemon,
    ) -> Result<(), SaveDataError>;
    fn box_count(&self) -> usize;
}
