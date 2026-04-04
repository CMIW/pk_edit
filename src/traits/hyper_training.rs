use crate::common::types::HyperTraining;
use crate::error::PokemonError;

pub trait HyperTrainingAccess {
    fn hyper_training(&self) -> Option<HyperTraining>;
    fn set_hyper_training(&mut self, training: HyperTraining) -> Result<(), PokemonError>;
}
