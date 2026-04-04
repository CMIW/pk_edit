use crate::common::types::{StorageType, Trainer};
use crate::error::SaveDataError;
use crate::traits::pokemon::Pokemon;

pub trait SaveFile {
    type Pokemon: Pokemon;
    fn party(&self) -> Result<Vec<Self::Pokemon>, SaveDataError>;
    fn trainer(&self) -> Result<Trainer, SaveDataError>;
    fn to_bytes(&self) -> Vec<u8>;
    fn swap_pokemon(
        &mut self,
        from: Self::Pokemon,
        from_storage: StorageType,
        to: Self::Pokemon,
        to_storage: StorageType,
    ) -> Result<(), SaveDataError>;
}
