use crate::common::types::Pocket;
use crate::error::SaveDataError;

pub trait BagAccess {
    fn read_pocket(&self, pocket: Pocket) -> Result<Vec<(String, u16)>, SaveDataError>;
    fn write_pocket(
        &mut self,
        pocket: Pocket,
        items: Vec<(String, u16)>,
    ) -> Result<(), SaveDataError>;
}
