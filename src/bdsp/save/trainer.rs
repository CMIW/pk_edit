use crate::common::types::{Gender, TimePlayed, Trainer, TrainerID};
use crate::error::SaveDataError;
use byteorder::{ByteOrder, LittleEndian};

const MY_STATUS_OFFSET: usize = 0x79BB4;
const OT_NAME_OFFSET: usize = MY_STATUS_OFFSET + 0x00;
const OT_NAME_LEN: usize = 0x1A;
const ID_OFFSET: usize = MY_STATUS_OFFSET + 0x1C;
const MONEY_OFFSET: usize = MY_STATUS_OFFSET + 0x20;
const GENDER_OFFSET: usize = MY_STATUS_OFFSET + 0x24;
const PLAY_TIME_OFFSET: usize = 0x79C04;

pub fn parse_trainer(save_data: &[u8]) -> Result<Trainer, SaveDataError> {
    let id_bytes = save_data
        .get(ID_OFFSET..ID_OFFSET + 4)
        .ok_or(SaveDataError::InvalidOffset(ID_OFFSET))?;
    let public = LittleEndian::read_u16(
        id_bytes
            .get(..2)
            .ok_or(SaveDataError::InvalidOffset(ID_OFFSET))?,
    );
    let private = LittleEndian::read_u16(
        id_bytes
            .get(2..4)
            .ok_or(SaveDataError::InvalidOffset(ID_OFFSET + 2))?,
    );

    let name_bytes = save_data
        .get(OT_NAME_OFFSET..OT_NAME_OFFSET + OT_NAME_LEN)
        .ok_or(SaveDataError::InvalidOffset(OT_NAME_OFFSET))?;
    let name = read_utf16le(name_bytes);

    let gender_byte = *save_data
        .get(GENDER_OFFSET)
        .ok_or(SaveDataError::InvalidOffset(GENDER_OFFSET))?;
    let gender = if gender_byte == 0 { Gender::F } else { Gender::M };

    let money = LittleEndian::read_u32(
        save_data
            .get(MONEY_OFFSET..MONEY_OFFSET + 4)
            .ok_or(SaveDataError::InvalidOffset(MONEY_OFFSET))?,
    );

    let time_bytes = save_data
        .get(PLAY_TIME_OFFSET..PLAY_TIME_OFFSET + 4)
        .ok_or(SaveDataError::InvalidOffset(PLAY_TIME_OFFSET))?;
    let time_played = TimePlayed {
        hours: LittleEndian::read_u16(
            time_bytes
                .get(..2)
                .ok_or(SaveDataError::InvalidOffset(PLAY_TIME_OFFSET))?,
        ),
        minutes: *time_bytes
            .get(2)
            .ok_or(SaveDataError::InvalidOffset(PLAY_TIME_OFFSET + 2))?,
        seconds: *time_bytes
            .get(3)
            .ok_or(SaveDataError::InvalidOffset(PLAY_TIME_OFFSET + 3))?,
        frames: 0,
    };

    Ok(Trainer {
        name,
        gender,
        id: TrainerID { public, private },
        time_played,
        money,
    })
}

pub fn write_trainer(save_data: &mut [u8], trainer: &Trainer) -> Result<(), SaveDataError> {
    let id_slice = save_data
        .get_mut(ID_OFFSET..ID_OFFSET + 4)
        .ok_or(SaveDataError::InvalidOffset(ID_OFFSET))?;
    LittleEndian::write_u16(
        id_slice
            .get_mut(..2)
            .ok_or(SaveDataError::InvalidOffset(ID_OFFSET))?,
        trainer.id.public,
    );
    LittleEndian::write_u16(
        id_slice
            .get_mut(2..4)
            .ok_or(SaveDataError::InvalidOffset(ID_OFFSET + 2))?,
        trainer.id.private,
    );

    let name_slice = save_data
        .get_mut(OT_NAME_OFFSET..OT_NAME_OFFSET + OT_NAME_LEN)
        .ok_or(SaveDataError::InvalidOffset(OT_NAME_OFFSET))?;
    name_slice.fill(0);
    write_utf16le(&trainer.name, name_slice);

    let gender_slice = save_data
        .get_mut(GENDER_OFFSET)
        .ok_or(SaveDataError::InvalidOffset(GENDER_OFFSET))?;
    *gender_slice = if trainer.gender == Gender::F { 0 } else { 1 };

    let money_slice = save_data
        .get_mut(MONEY_OFFSET..MONEY_OFFSET + 4)
        .ok_or(SaveDataError::InvalidOffset(MONEY_OFFSET))?;
    LittleEndian::write_u32(money_slice, trainer.money);

    let time_slice = save_data
        .get_mut(PLAY_TIME_OFFSET..PLAY_TIME_OFFSET + 4)
        .ok_or(SaveDataError::InvalidOffset(PLAY_TIME_OFFSET))?;
    LittleEndian::write_u16(
        time_slice
            .get_mut(..2)
            .ok_or(SaveDataError::InvalidOffset(PLAY_TIME_OFFSET))?,
        trainer.time_played.hours,
    );
    *time_slice
        .get_mut(2)
        .ok_or(SaveDataError::InvalidOffset(PLAY_TIME_OFFSET + 2))? = trainer.time_played.minutes;
    *time_slice
        .get_mut(3)
        .ok_or(SaveDataError::InvalidOffset(PLAY_TIME_OFFSET + 3))? = trainer.time_played.seconds;

    Ok(())
}

fn read_utf16le(bytes: &[u8]) -> String {
    let mut units = Vec::new();
    for chunk in bytes.chunks_exact(2) {
        let unit = LittleEndian::read_u16(chunk);
        if unit == 0 {
            break;
        }
        units.push(unit);
    }
    char::decode_utf16(units)
        .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect()
}

fn write_utf16le(text: &str, bytes: &mut [u8]) {
    for (i, unit) in text.encode_utf16().take(bytes.len() / 2).enumerate() {
        if let Some(slice) = bytes.get_mut(i * 2..i * 2 + 2) {
            LittleEndian::write_u16(slice, unit);
        }
    }
}
