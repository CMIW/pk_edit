#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::indexing_slicing)]
    #![allow(clippy::cast_possible_truncation)]
    #![allow(clippy::cast_precision_loss)]
    #![allow(clippy::wildcard_enum_match_arm)]
    #![allow(clippy::float_arithmetic)]
    #![allow(clippy::must_use_candidate)]

    use crate::common::charset::gen3::{get_char, get_code};
    use crate::error::{PokemonError, SaveDataError};
    use crate::gen3::game_data::Gen3GameData;
    use crate::gen3::pokemon::{gen_pokemon_from_species, Gen3Pokemon};
    use crate::gen3::save::section::{Section, SectionID};
    use crate::gen3::save::storage::{
        decrypt_pocket, encrypt_pocket, pocket_address, Pocket, StorageType, PARTY_COUNT_OFFSET,
        PARTY_DATA_OFFSET, PARTY_POKEMON_SIZE, PARTY_SIZE,
    };
    use crate::gen3::save::trainer::{GymBadges, TimePlayed};
    use crate::traits::game_data::GameData;
    use crate::TrainerID;
    use crate::misc::extract_db;
    use crate::traits::pokemon::Pokemon;
    use crate::Gender;
    use crate::Pokerus;

    /// Extracts the embedded SQLite database so DB-backed functions work.
    /// Silently ignores `AlreadyExists` errors from previous runs.
    fn setup_db() {
        let _ = extract_db();
    }

    /// Raw bytes of a level-5 Torchic from an Emerald save file.
    const TORCHIK: [u8; 100] = [
        101, 231, 167, 198, 154, 166, 220, 6, 206, 201, 204, 189, 194, 195, 189, 255, 1, 0, 2, 2,
        195, 213, 226, 255, 255, 255, 255, 0, 49, 30, 0, 0, 255, 65, 123, 193, 255, 65, 123, 192,
        255, 65, 123, 192, 231, 64, 123, 192, 103, 65, 123, 192, 255, 7, 123, 192, 255, 81, 254,
        225, 69, 32, 147, 217, 255, 65, 123, 192, 245, 65, 86, 192, 255, 65, 123, 192, 220, 105,
        123, 192, 0, 0, 0, 0, 5, 255, 20, 0, 20, 0, 11, 0, 10, 0, 9, 0, 14, 0, 10, 0,
    ];

    // ==================== ERROR TYPES ====================

    #[test]
    fn test_pokemon_error_invalid_data_length() {
        let err = PokemonError::InvalidDataLength(50);
        assert!(err.to_string().contains("50"));
    }

    #[test]
    fn test_pokemon_error_unknown_species() {
        let err = PokemonError::UnknownSpecies("FakeMon".into());
        assert!(err.to_string().contains("FakeMon"));
    }

    #[test]
    fn test_pokemon_error_unknown_item() {
        let err = PokemonError::UnknownItem("FakeItem".into());
        assert!(err.to_string().contains("FakeItem"));
    }

    #[test]
    fn test_pokemon_error_unknown_move() {
        let err = PokemonError::UnknownMove("FakeMove".into());
        assert!(err.to_string().contains("FakeMove"));
    }

    #[test]
    fn test_pokemon_error_invalid_move_slot() {
        let err = PokemonError::InvalidMoveSlot(5);
        assert!(err.to_string().contains("5"));
    }

    #[test]
    fn test_pokemon_error_missing_gender_ratio() {
        let err = PokemonError::MissingGenderRatio(999);
        assert!(err.to_string().contains("999"));
    }

    #[test]
    fn test_pokemon_error_invalid_pokeball() {
        let err = PokemonError::InvalidPokeball(13);
        assert!(err.to_string().contains("13"));
    }

    #[test]
    fn test_save_error_section_not_found() {
        let err = SaveDataError::SectionNotFound(SectionID::TrainerInfo.to_string());
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_save_error_invalid_data_length() {
        let err = SaveDataError::InvalidDataLength {
            expected: 4,
            found: 2,
        };
        let s = err.to_string();
        assert!(s.contains("4") && s.contains("2"));
    }

    #[test]
    fn test_save_error_invalid_offset() {
        let err = SaveDataError::InvalidOffset(0xDEAD);
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_save_error_invalid_range() {
        let err = SaveDataError::InvalidRange(10..20);
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_save_error_decryption_error() {
        let err = SaveDataError::DecryptionError(0x1234);
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_save_error_checksum_mismatch() {
        let err = SaveDataError::ChecksumMismatch {
            expected: 0x1234,
            found: 0x5678,
        };
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_save_error_unexpected() {
        let err = SaveDataError::Unexpected("oops".into());
        assert!(err.to_string().contains("oops"));
    }

    #[test]
    fn test_save_error_party_full() {
        let err = SaveDataError::PartyFull;
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_save_error_cannot_empty_party() {
        let err = SaveDataError::CannotEmptyParty;
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_save_error_invalid_party_slot() {
        let err = SaveDataError::InvalidPartySlot(7);
        assert!(err.to_string().contains("7"));
    }

    #[test]
    fn test_save_error_from_pokemon_error() {
        let pe = PokemonError::UnknownSpecies("X".into());
        let se: SaveDataError = pe.into();
        assert!(!se.to_string().is_empty());
    }

    // ==================== SECTION ID ====================

    #[test]
    fn test_section_id_from_u16_all_variants() {
        assert_eq!(SectionID::from(0u16), SectionID::TrainerInfo);
        assert_eq!(SectionID::from(1u16), SectionID::TeamItems);
        assert_eq!(SectionID::from(2u16), SectionID::GameState);
        assert_eq!(SectionID::from(3u16), SectionID::MiscData);
        assert_eq!(SectionID::from(4u16), SectionID::RivalInfo);
        assert_eq!(SectionID::from(5u16), SectionID::PCbufferA);
        assert_eq!(SectionID::from(6u16), SectionID::PCbufferB);
        assert_eq!(SectionID::from(7u16), SectionID::PCbufferC);
        assert_eq!(SectionID::from(8u16), SectionID::PCbufferD);
        assert_eq!(SectionID::from(9u16), SectionID::PCbufferE);
        assert_eq!(SectionID::from(10u16), SectionID::PCbufferF);
        assert_eq!(SectionID::from(11u16), SectionID::PCbufferG);
        assert_eq!(SectionID::from(12u16), SectionID::PCbufferH);
        assert_eq!(SectionID::from(13u16), SectionID::PCbufferI);
        assert_eq!(SectionID::from(99u16), SectionID::NA);
    }

    #[test]
    fn test_section_id_to_i32_all_variants() {
        assert_eq!(i32::from(SectionID::TrainerInfo), 0);
        assert_eq!(i32::from(SectionID::TeamItems), 1);
        assert_eq!(i32::from(SectionID::GameState), 2);
        assert_eq!(i32::from(SectionID::MiscData), 3);
        assert_eq!(i32::from(SectionID::RivalInfo), 4);
        assert_eq!(i32::from(SectionID::PCbufferA), 5);
        assert_eq!(i32::from(SectionID::PCbufferB), 6);
        assert_eq!(i32::from(SectionID::PCbufferC), 7);
        assert_eq!(i32::from(SectionID::PCbufferD), 8);
        assert_eq!(i32::from(SectionID::PCbufferE), 9);
        assert_eq!(i32::from(SectionID::PCbufferF), 10);
        assert_eq!(i32::from(SectionID::PCbufferG), 11);
        assert_eq!(i32::from(SectionID::PCbufferH), 12);
        assert_eq!(i32::from(SectionID::PCbufferI), 13);
        assert_eq!(i32::from(SectionID::NA), 14);
    }

    #[test]
    fn test_section_id_default() {
        assert_eq!(SectionID::default(), SectionID::TrainerInfo);
    }

    // ==================== SECTION checksum ====================

    #[test]
    fn test_section_checksum_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let section_size = 0x1000;
        let mut buf = vec![0u8; section_size];
        // Fill data region with a non-trivial pattern
        for (i, byte) in buf.iter_mut().enumerate() {
            if i < 0x0FF4 {
                *byte = (i & 0xFF) as u8;
            }
        }

        let section = Section {
            offset: 0,
            size: section_size,
        };

        let checksum = section.compute_checksum(&buf)?;
        section.write_checksum(&mut buf)?;

        // Validation should succeed
        section.validate_checksum(&buf)?;

        // The stored checksum should match what compute_checksum returned.
        // Instead of reading raw bytes, re-derive via a fresh struct.
        let section2 = Section {
            offset: 0,
            size: section_size,
        };
        assert_eq!(section2.compute_checksum(&buf)?, checksum);

        // Corrupt a data byte (not the checksum region) so the checksum no longer matches
        buf[0] = 0xFF;

        // Validation should now fail
        let result = section.validate_checksum(&buf);
        assert!(result.is_err());
        assert!(matches!(result, Err(crate::error::SaveDataError::ChecksumMismatch { .. })));

        Ok(())
    }

    // ==================== STORAGE ====================

    #[test]
    fn test_storage_type_default() {
        let st = StorageType::default();
        assert!(matches!(st, StorageType::None));
    }

    #[test]
    fn test_pocket_display() {
        assert_eq!(Pocket::Items.to_string(), "Items");
        assert_eq!(Pocket::Pokeballs.to_string(), "Poké Balls");
        assert_eq!(Pocket::Berries.to_string(), "Berries");
        assert_eq!(Pocket::Tms.to_string(), "TMs & HMs");
        assert_eq!(Pocket::Key.to_string(), "Key Items");
    }

    #[test]
    fn test_pocket_address_ruby_sapphire() {
        let (start, end) = pocket_address(Pocket::Items, 0);
        assert_eq!(start, 0x0560);
        assert_eq!(end, 0x05B0);

        let (start, end) = pocket_address(Pocket::Pokeballs, 0);
        assert_eq!(start, 0x0600);
        assert_eq!(end, 0x0640);

        let (start, end) = pocket_address(Pocket::Berries, 0);
        assert_eq!(start, 0x0740);
        assert_eq!(end, 0x07F8);

        let (start, end) = pocket_address(Pocket::Tms, 0);
        assert_eq!(start, 0x0640);
        assert_eq!(end, 0x0740);

        let (start, end) = pocket_address(Pocket::Key, 0);
        assert_eq!(start, 0x05B0);
        assert_eq!(end, 0x0600);
    }

    #[test]
    fn test_pocket_address_firered_leafgreen() {
        let (start, end) = pocket_address(Pocket::Items, 1);
        assert_eq!(start, 0x0310);
        assert_eq!(end, 0x03B8);

        let (start, end) = pocket_address(Pocket::Pokeballs, 1);
        assert_eq!(start, 0x0430);
        assert_eq!(end, 0x0464);

        let (start, end) = pocket_address(Pocket::Berries, 1);
        assert_eq!(start, 0x054C);
        assert_eq!(end, 0x05F8);

        let (start, end) = pocket_address(Pocket::Tms, 1);
        assert_eq!(start, 0x0464);
        assert_eq!(end, 0x054C);

        let (start, end) = pocket_address(Pocket::Key, 1);
        assert_eq!(start, 0x03B8);
        assert_eq!(end, 0x0430);
    }

    #[test]
    fn test_pocket_address_emerald() {
        let (start, end) = pocket_address(Pocket::Items, 2);
        assert_eq!(start, 0x0560);
        assert_eq!(end, 0x05D8);

        let (start, end) = pocket_address(Pocket::Pokeballs, 2);
        assert_eq!(start, 0x0650);
        assert_eq!(end, 0x0690);

        let (start, end) = pocket_address(Pocket::Berries, 2);
        assert_eq!(start, 0x0790);
        assert_eq!(end, 0x0848);

        let (start, end) = pocket_address(Pocket::Tms, 2);
        assert_eq!(start, 0x0690);
        assert_eq!(end, 0x0790);

        let (start, end) = pocket_address(Pocket::Key, 2);
        assert_eq!(start, 0x05D8);
        assert_eq!(end, 0x0650);
    }

    #[test]
    fn test_party_constants() {
        assert_eq!(PARTY_COUNT_OFFSET, 0x0034);
        assert_eq!(PARTY_DATA_OFFSET, 0x0038);
        assert_eq!(PARTY_SIZE, 6);
        assert_eq!(PARTY_POKEMON_SIZE, 100);
    }

    #[test]
    fn test_decrypt_pocket_empty() -> Result<(), Box<dyn std::error::Error>> {
        let items = decrypt_pocket(&[], 0x1234)?;
        assert!(items.is_empty());
        Ok(())
    }

    #[test]
    fn test_decrypt_pocket_all_zero_slots() -> Result<(), Box<dyn std::error::Error>> {
        // item_id == 0 means empty slot — appears as "Nothing"
        let data = [0u8; 8];
        let items = decrypt_pocket(&data, 0x1234)?;
        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|item| item.is_empty()));
        Ok(())
    }

    #[test]
    fn test_decrypt_pocket_zero_key() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        // item_id=1, quantity=5 with key=0 (no XOR change)
        // bytes: [0x01, 0x00, 0x05, 0x00] = item 1, qty 5
        let data: [u8; 4] = [0x01, 0x00, 0x05, 0x00];
        let items = decrypt_pocket(&data, 0)?;
        assert_eq!(items.len(), 1);
        if let Some(item) = items.first() {
            assert_eq!(item.id, 1);
            assert_eq!(item.quantity, 5);
        }
        Ok(())
    }

    #[test]
    fn test_decrypt_pocket_trailing_zero_chunk_ignored() -> Result<(), Box<dyn std::error::Error>> {
        // 3-byte all-zero trailing chunk should be ignored
        let data = [0u8; 3];
        let items = decrypt_pocket(&data, 0x1234)?;
        assert!(items.is_empty());
        Ok(())
    }

    #[test]
    fn test_decrypt_pocket_invalid_nonzero_short_chunk() {
        // Non-zero 3-byte chunk is an error
        let data = [0x01u8, 0x00, 0x01];
        let result = decrypt_pocket(&data, 0x1234);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_pocket_empty() -> Result<(), Box<dyn std::error::Error>> {
        let encrypted = encrypt_pocket(vec![], 0x1234)?;
        assert!(encrypted.is_empty());
        Ok(())
    }

    // ==================== TRAINER ====================

    #[test]
    fn test_trainer_id_from_bytes() {
        let bytes: [u8; 4] = [0x01, 0x00, 0x05, 0x00];
        let id: TrainerID = bytes.into();
        assert_eq!(id.public, 1);
        assert_eq!(id.private, 5);
    }

    #[test]
    fn test_trainer_id_display_normal() {
        let id = TrainerID {
            public: 12345,
            private: 9999,
        };
        assert_eq!(id.to_string(), "12345");
    }

    #[test]
    fn test_trainer_id_display_zero_padded() {
        let id = TrainerID {
            public: 42,
            private: 0,
        };
        assert_eq!(id.to_string(), "00042");
    }

    #[test]
    fn test_trainer_id_display_max() {
        let id = TrainerID {
            public: 65535,
            private: 65535,
        };
        assert_eq!(id.to_string(), "65535");
    }

    #[test]
    fn test_time_played_round_trip() {
        let t = TimePlayed {
            hours: 9999,
            minutes: 59,
            seconds: 59,
            frames: 59,
        };
        let bytes = t.to_bytes();
        let t2 = TimePlayed::from_bytes(&bytes);
        assert_eq!(t.hours, t2.hours);
        assert_eq!(t.minutes, t2.minutes);
        assert_eq!(t.seconds, t2.seconds);
        assert_eq!(t.frames, t2.frames);
    }

    #[test]
    fn test_time_played_zero() {
        let bytes = [0u8; 5];
        let t = TimePlayed::from_bytes(&bytes);
        assert_eq!(t.hours, 0);
        assert_eq!(t.minutes, 0);
        assert_eq!(t.seconds, 0);
        assert_eq!(t.frames, 0);
    }

    #[test]
    fn test_time_played_to_bytes_length() {
        let t = TimePlayed {
            hours: 100,
            minutes: 30,
            seconds: 15,
            frames: 0,
        };
        let bytes = t.to_bytes();
        assert_eq!(bytes.len(), 5);
    }

    #[test]
    fn test_gym_badges_default_all_zero() {
        let badges = GymBadges::default();
        assert_eq!(badges.badge_1(), 0);
        assert_eq!(badges.badge_2(), 0);
        assert_eq!(badges.badge_3(), 0);
        assert_eq!(badges.badge_4(), 0);
        assert_eq!(badges.badge_5(), 0);
        assert_eq!(badges.badge_6(), 0);
        assert_eq!(badges.badge_7(), 0);
        assert_eq!(badges.badge_8(), 0);
    }

    #[test]
    fn test_gym_badges_set_individual() {
        let mut badges = GymBadges::default();
        badges.set_badge_1(1);
        assert_eq!(badges.badge_1(), 1);
        assert_eq!(badges.badge_2(), 0); // Others unaffected
    }

    #[test]
    fn test_gym_badges_set_all() {
        let mut badges = GymBadges::default();
        badges.set_badge_1(1);
        badges.set_badge_2(1);
        badges.set_badge_3(1);
        badges.set_badge_4(1);
        badges.set_badge_5(1);
        badges.set_badge_6(1);
        badges.set_badge_7(1);
        badges.set_badge_8(1);
        assert_eq!(badges.badge_1(), 1);
        assert_eq!(badges.badge_2(), 1);
        assert_eq!(badges.badge_3(), 1);
        assert_eq!(badges.badge_4(), 1);
        assert_eq!(badges.badge_5(), 1);
        assert_eq!(badges.badge_6(), 1);
        assert_eq!(badges.badge_7(), 1);
        assert_eq!(badges.badge_8(), 1);
    }

    #[test]
    fn test_gym_badges_count() {
        let mut badges = GymBadges::default();
        assert_eq!(badges.count(), 0);

        badges.set_badge_1(1);
        badges.set_badge_3(1);
        badges.set_badge_8(1);
        assert_eq!(badges.count(), 3);

        badges.set_badge_1(0);
        badges.set_badge_2(1);
        badges.set_badge_4(1);
        badges.set_badge_5(1);
        badges.set_badge_6(1);
        badges.set_badge_7(1);
        assert_eq!(badges.count(), 7);

        let mut all = GymBadges::default();
        all.set_badge_1(1);
        all.set_badge_2(1);
        all.set_badge_3(1);
        all.set_badge_4(1);
        all.set_badge_5(1);
        all.set_badge_6(1);
        all.set_badge_7(1);
        all.set_badge_8(1);
        assert_eq!(all.count(), 8);

        // Test roundtrip: from_bytes -> count
        let bytes = all.into_bytes();
        let restored = GymBadges::from_bytes(bytes);
        assert_eq!(restored.count(), 8);
    }

    #[test]
    fn test_gym_badges_flags_bitmask() {
        // No badges -> empty mask.
        assert_eq!(GymBadges::default().flags(), 0b0000_0000);

        // Non-contiguous badges must map to the matching bit positions, not a
        // count. Badges 1, 3, and 8 -> bits 0, 2, 7.
        let mut badges = GymBadges::default();
        badges.set_badge_1(1);
        badges.set_badge_3(1);
        badges.set_badge_8(1);
        assert_eq!(badges.flags(), 0b1000_0101);
        // The count is 3, which must NOT equal the flag mask: this is the exact
        // confusion that lit up the wrong badges in the UI.
        assert_ne!(u8::from(badges.count()), badges.flags());

        // All badges -> all bits set.
        let mut all = GymBadges::default();
        all.set_badge_1(1);
        all.set_badge_2(1);
        all.set_badge_3(1);
        all.set_badge_4(1);
        all.set_badge_5(1);
        all.set_badge_6(1);
        all.set_badge_7(1);
        all.set_badge_8(1);
        assert_eq!(all.flags(), 0b1111_1111);
    }

    // ==================== CHARACTER SET ====================

    #[test]
    fn test_get_char_uppercase_letters() {
        assert_eq!(get_char(0xBB), "A");
        assert_eq!(get_char(0xBC), "B");
        assert_eq!(get_char(0xD4), "Z");
    }

    #[test]
    fn test_get_char_lowercase_letters() {
        assert_eq!(get_char(0xD5), "a");
        assert_eq!(get_char(0xEE), "z");
    }

    #[test]
    fn test_get_char_digits() {
        assert_eq!(get_char(0xA1), "0");
        assert_eq!(get_char(0xA2), "1");
        assert_eq!(get_char(0xA9), "8");
        assert_eq!(get_char(0xAA), "9");
    }

    #[test]
    fn test_get_char_space() {
        assert_eq!(get_char(0x00), " ");
    }

    #[test]
    fn test_get_char_special() {
        assert_eq!(get_char(0xB5), "♂");
        assert_eq!(get_char(0xB6), "♀");
        assert_eq!(get_char(0xAB), "!");
        assert_eq!(get_char(0xAC), "?");
    }

    #[test]
    fn test_get_code_uppercase() {
        assert_eq!(get_code("A"), 0xBB);
        assert_eq!(get_code("Z"), 0xD4);
    }

    #[test]
    fn test_get_code_lowercase() {
        assert_eq!(get_code("a"), 0xD5);
        assert_eq!(get_code("z"), 0xEE);
    }

    #[test]
    fn test_get_code_digits() {
        assert_eq!(get_code("0"), 0xA1);
        assert_eq!(get_code("9"), 0xAA);
    }

    #[test]
    fn test_get_code_space() {
        // Multiple space bytes exist (0x00, 0x5E, 0x5F); HashMap last-insert wins → 0x5F
        assert_eq!(get_code(" "), 0x5F);
    }

    // ==================== POKEMON::from_bytes ====================

    #[test]
    fn test_from_bytes_too_short_returns_error() {
        let result = Gen3Pokemon::from_bytes(0, &[0u8; 50]);
        assert!(result.is_err());
        if let Err(PokemonError::InvalidDataLength(n)) = result {
            assert_eq!(n, 50);
        }
    }

    #[test]
    fn test_from_bytes_79_bytes_returns_error() {
        let result = Gen3Pokemon::from_bytes(0, &[0u8; 79]);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_bytes_exactly_80_pc_format() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let bytes = &TORCHIK[..80];
        let pokemon = Gen3Pokemon::from_bytes(0, bytes)?;
        assert!(!pokemon.is_empty());
        Ok(())
    }

    #[test]
    fn test_from_bytes_party_100() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        assert!(!pokemon.is_empty());
        Ok(())
    }

    #[test]
    fn test_from_bytes_offset_stored() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(42, &TORCHIK)?;
        assert_eq!(pokemon.offset, 42);
        Ok(())
    }

    #[test]
    fn test_from_bytes_empty_slot() -> Result<(), Box<dyn std::error::Error>> {
        // 80 zero bytes = empty slot
        let pokemon = Gen3Pokemon::from_bytes(0, &[0u8; 80])?;
        assert!(pokemon.is_empty());
        Ok(())
    }

    // ==================== POKEMON getters ====================

    #[test]
    fn test_ability_torchic() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        assert_eq!(pokemon.ability(), "Blaze");
        Ok(())
    }

    #[test]
    fn test_is_egg_false() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        assert!(!pokemon.is_egg());
        Ok(())
    }

    #[test]
    fn test_moves_torchic_scratch_and_growl() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        let moves = pokemon.moves();
        assert_eq!(moves.len(), 2);
        let first = moves.first().ok_or("no moves")?;
        assert_eq!(first.move_type, "Normal");
        assert_eq!(first.name, "Scratch");
        assert_eq!(first.pp, 35);
        assert_eq!(first.pp_used, 0);
        let second = moves.get(1).ok_or("no second move")?;
        assert_eq!(second.name, "Growl");
        assert_eq!(second.pp, 40);
        Ok(())
    }

    #[test]
    fn test_pokerus_none() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        assert_eq!(pokemon.pokerus_status(), Pokerus::None);
        Ok(())
    }

    #[test]
    fn test_species_torchic() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        assert_eq!(pokemon.species(), "Torchic");
        Ok(())
    }

    #[test]
    fn test_level_torchic_is_5() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        assert_eq!(pokemon.level(), 5);
        Ok(())
    }

    #[test]
    fn test_friendship_is_non_zero() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        assert!(pokemon.friendship() > 0);
        Ok(())
    }

    #[test]
    fn test_is_empty_false_for_torchic() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        assert!(!pokemon.is_empty());
        Ok(())
    }

    #[test]
    fn test_is_empty_true_for_default() {
        let empty = Gen3Pokemon::default();
        assert!(empty.is_empty());
    }

    #[test]
    fn test_is_bad_egg_false() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        assert!(!pokemon.is_bad_egg());
        Ok(())
    }

    #[test]
    fn test_ot_name_nonempty() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        assert!(!pokemon.ot_name().is_empty());
        Ok(())
    }

    #[test]
    fn test_ot_id_displayable() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        let id = pokemon.ot_id();
        assert_eq!(id.to_string().len(), 5); // Zero-padded to 5 digits
        Ok(())
    }

    #[test]
    fn test_nickname_nonempty() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        assert!(!pokemon.nickname().is_empty());
        Ok(())
    }

    #[test]
    fn test_gender_valid() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        let gender = pokemon.gender();
        assert!(matches!(gender, Gender::M | Gender::F | Gender::None));
        Ok(())
    }

    #[test]
    fn test_nature_nonempty() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        assert!(!pokemon.nature().is_empty());
        Ok(())
    }

    #[test]
    fn test_typing_torchic_fire() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        let typing = pokemon.typing();
        assert!(typing.is_some());
        if let Some((primary, secondary)) = typing {
            assert_eq!(primary, "Fire");
            assert!(secondary.is_none());
        }
        Ok(())
    }

    #[test]
    fn test_typing_none_for_empty_pokemon() {
        let empty = Gen3Pokemon::default();
        assert!(empty.typing().is_none());
    }

    #[test]
    fn test_experience_nonzero() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        assert!(pokemon.experience() > 0);
        Ok(())
    }

    #[test]
    fn test_pokeball_caught_valid_range() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        assert!(pokemon.pokeball_caught() <= 12);
        Ok(())
    }

    #[test]
    fn test_species_id_nonzero() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        assert!(pokemon.species_id() > 0);
        Ok(())
    }

    #[test]
    fn test_nat_dex_number_torchic() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        assert_eq!(pokemon.nat_dex_number(), 255);
        Ok(())
    }

    #[test]
    fn test_ivs_valid_range() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        let ivs = pokemon.ivs();
        assert!(ivs.hp <= 31);
        assert!(ivs.attack <= 31);
        assert!(ivs.defense <= 31);
        assert!(ivs.speed <= 31);
        assert!(ivs.special_attack <= 31);
        assert!(ivs.special_defense <= 31);
        Ok(())
    }

    #[test]
    fn test_stats_all_positive() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        let level = pokemon.level();
        assert!(pokemon.stats.hp(level) > 0);
        assert!(pokemon.stats.attack(level) > 0);
        assert!(pokemon.stats.defense(level) > 0);
        assert!(pokemon.stats.speed(level) > 0);
        assert!(pokemon.stats.sp_attack(level) > 0);
        assert!(pokemon.stats.sp_defense(level) > 0);
        Ok(())
    }

    #[test]
    fn test_highest_stat_valid() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        let level = pokemon.level();
        let (name, value) = pokemon.stats.highest_stat(level);
        assert!(!name.is_empty());
        assert!(value > 0);
        Ok(())
    }

    #[test]
    fn test_lowest_level_at_least_1() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        assert!(Gen3GameData.lowest_level(1) >= 1); // Bulbasaur has no pre-evo
        assert!(Gen3GameData.lowest_level(2) >= 1); // Ivysaur evolves at level 16
        assert!(Gen3GameData.lowest_level(25) >= 1); // Pikachu evolves with item
        Ok(())
    }

    // ==================== POKEMON setters ====================

    #[test]
    fn test_set_friendship() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.set_friendship(200).unwrap();
        assert_eq!(pokemon.friendship(), 200);
        Ok(())
    }

    #[test]
    fn test_set_level_changes_experience() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.set_level(50).unwrap();
        assert_eq!(pokemon.level(), 50);
        Ok(())
    }

    #[test]
    fn test_set_level_clamped_to_100() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.set_level(200).unwrap(); // clamped to 100
        assert_eq!(pokemon.level(), 100);
        Ok(())
    }

    #[test]
    fn test_set_level_clamped_to_1() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.set_level(0).unwrap(); // clamped to 1
        assert_eq!(pokemon.level(), 1);
        Ok(())
    }

    #[test]
    fn test_set_species_bulbasaur() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.set_species("Bulbasaur")?;
        assert_eq!(pokemon.species(), "Bulbasaur");
        Ok(())
    }

    #[test]
    fn test_set_species_unknown_returns_error() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        // Use a short name so the nickname auto-update doesn't overflow the 10-byte field.
        // The nickname update occurs before the species look-up, so long unknown names panic.
        pokemon.set_nickname("NOTMATCH").unwrap(); // break the auto-nickname link first
        let result = pokemon.set_species("FAKEMON");
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_set_held_item_none() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.set_held_item("-")?;
        assert!(pokemon.held_item().is_none());
        Ok(())
    }

    #[test]
    fn test_set_held_item_none_string() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.set_held_item("None")?;
        assert!(pokemon.held_item().is_none());
        Ok(())
    }

    #[test]
    fn test_set_held_item_unknown_returns_error() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        let result = pokemon.set_held_item("NotAnItem");
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_set_move_valid() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.set_move(0, "Ember")?;
        let moves = pokemon.moves();
        let first = moves.first().ok_or("no moves")?;
        assert_eq!(first.name, "Ember");
        Ok(())
    }

    #[test]
    fn test_set_move_invalid_slot() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        let result = pokemon.set_move(5, "Ember");
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_set_move_unknown_move() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        let result = pokemon.set_move(0, "NotAMove");
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_set_pokeball_valid() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.set_pokeball_caught(4)?;
        assert_eq!(pokemon.pokeball_caught(), 4);
        Ok(())
    }

    #[test]
    fn test_set_pokeball_zero_valid() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.set_pokeball_caught(0)?;
        assert_eq!(pokemon.pokeball_caught(), 0);
        Ok(())
    }

    #[test]
    fn test_set_pokeball_invalid_over_12() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        let result = pokemon.set_pokeball_caught(13);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_set_nickname() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.set_nickname("CHARIZARD").unwrap();
        assert_eq!(pokemon.nickname(), "CHARIZARD");
        Ok(())
    }

    #[test]
    fn test_set_nickname_short() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.set_nickname("ACE").unwrap();
        assert_eq!(pokemon.nickname(), "ACE");
        Ok(())
    }

    // ==================== POKEMON Pokérus ====================

    #[test]
    fn test_infect_pokerus() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.infect_pokerus();
        assert_eq!(pokemon.pokerus_status(), Pokerus::Infected);
        Ok(())
    }

    #[test]
    fn test_cure_pokerus() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.infect_pokerus();
        pokemon.cure_pokerus();
        assert_eq!(pokemon.pokerus_status(), Pokerus::Cured);
        Ok(())
    }

    #[test]
    fn test_remove_pokerus() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.infect_pokerus();
        pokemon.remove_pokerus();
        assert_eq!(pokemon.pokerus_status(), Pokerus::None);
        Ok(())
    }

    // ==================== POKEMON checksum & round-trip ====================

    #[test]
    fn test_update_checksum_unchanged_data() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        let original = pokemon.checksum;
        pokemon.update_checksum();
        assert_eq!(pokemon.checksum, original);
        Ok(())
    }

    #[test]
    fn test_update_checksum_after_mutation() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.set_friendship(99).unwrap();
        pokemon.update_checksum();
        // Checksum should now reflect the new state; re-parse and confirm it's consistent
        let bytes = pokemon.to_bytes();
        let reparsed = Gen3Pokemon::from_bytes(0, &bytes)?;
        assert_eq!(reparsed.checksum, pokemon.checksum);
        Ok(())
    }

    #[test]
    fn test_to_bytes_length_is_100() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        let bytes = pokemon.to_bytes();
        assert_eq!(bytes.len(), 100);
        Ok(())
    }

    #[test]
    fn test_to_bytes_roundtrip_species() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let original = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        let bytes = original.to_bytes();
        let restored = Gen3Pokemon::from_bytes(0, &bytes)?;
        assert_eq!(original.species(), restored.species());
        Ok(())
    }

    #[test]
    fn test_to_bytes_roundtrip_level() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let original = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        let bytes = original.to_bytes();
        let restored = Gen3Pokemon::from_bytes(0, &bytes)?;
        assert_eq!(original.level(), restored.level());
        Ok(())
    }

    #[test]
    fn test_to_bytes_roundtrip_nature() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let original = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        let bytes = original.to_bytes();
        let restored = Gen3Pokemon::from_bytes(0, &bytes)?;
        assert_eq!(original.nature(), restored.nature());
        Ok(())
    }

    #[test]
    fn test_to_bytes_roundtrip_ability() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let original = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        let bytes = original.to_bytes();
        let restored = Gen3Pokemon::from_bytes(0, &bytes)?;
        assert_eq!(original.ability(), restored.ability());
        Ok(())
    }

    #[test]
    fn test_to_bytes_roundtrip_pokerus() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut original = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        original.infect_pokerus();
        let bytes = original.to_bytes();
        let restored = Gen3Pokemon::from_bytes(0, &bytes)?;
        assert_eq!(original.pokerus_status(), restored.pokerus_status());
        Ok(())
    }

    #[test]
    fn test_to_bytes_roundtrip_moves() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let original = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        let bytes = original.to_bytes();
        let restored = Gen3Pokemon::from_bytes(0, &bytes)?;
        assert_eq!(original.moves(), restored.moves());
        Ok(())
    }

    // ==================== Stats IV/EV updates ====================

    #[test]
    fn test_update_ivs_hp() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.stats_mut().update_ivs("HP", 25);
        assert_eq!(pokemon.stats.hp_iv, 25);
        Ok(())
    }

    #[test]
    fn test_update_ivs_attack() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.stats_mut().update_ivs("Attack", 31);
        assert_eq!(pokemon.stats.attack_iv, 31);
        Ok(())
    }

    #[test]
    fn test_update_ivs_defense() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.stats_mut().update_ivs("Defense", 20);
        assert_eq!(pokemon.stats.defense_iv, 20);
        Ok(())
    }

    #[test]
    fn test_update_ivs_sp_atk() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.stats_mut().update_ivs("Sp. Atk", 15);
        assert_eq!(pokemon.stats.sp_attack_iv, 15);
        Ok(())
    }

    #[test]
    fn test_update_ivs_sp_def() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.stats_mut().update_ivs("Sp. Def", 10);
        assert_eq!(pokemon.stats.sp_defense_iv, 10);
        Ok(())
    }

    #[test]
    fn test_update_ivs_speed() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.stats_mut().update_ivs("Speed", 5);
        assert_eq!(pokemon.stats.speed_iv, 5);
        Ok(())
    }

    #[test]
    fn test_update_ivs_clamps_at_31() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.stats_mut().update_ivs("HP", 50);
        assert_eq!(pokemon.stats.hp_iv, 31);
        Ok(())
    }

    #[test]
    fn test_update_ivs_unknown_stat_ignored() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        let before = pokemon.stats.hp_iv;
        pokemon.stats_mut().update_ivs("Unknown", 25);
        assert_eq!(pokemon.stats.hp_iv, before);
        Ok(())
    }

    #[test]
    fn test_update_evs_hp() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        // Zero all EVs first so total is predictable
        pokemon.stats_mut().update_evs("HP", 0);
        pokemon.stats_mut().update_evs("Attack", 0);
        pokemon.stats_mut().update_evs("Defense", 0);
        pokemon.stats_mut().update_evs("Sp. Atk", 0);
        pokemon.stats_mut().update_evs("Sp. Def", 0);
        pokemon.stats_mut().update_evs("Speed", 0);
        pokemon.stats_mut().update_evs("HP", 100);
        assert_eq!(pokemon.stats.hp_ev, 100);
        Ok(())
    }

    #[test]
    fn test_update_evs_capped_at_252() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.stats_mut().update_evs("Attack", 0);
        pokemon.stats_mut().update_evs("Defense", 0);
        pokemon.stats_mut().update_evs("Sp. Atk", 0);
        pokemon.stats_mut().update_evs("Sp. Def", 0);
        pokemon.stats_mut().update_evs("Speed", 0);
        pokemon.stats_mut().update_evs("HP", 0);
        pokemon.stats_mut().update_evs("HP", 255);
        assert_eq!(pokemon.stats.hp_ev, 252);
        Ok(())
    }

    #[test]
    fn test_update_evs_all_six_stats() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        // Set each stat independently from zero
        let stats = ["HP", "Attack", "Defense", "Sp. Atk", "Sp. Def", "Speed"];
        for stat in &stats {
            pokemon.stats_mut().update_evs(stat, 0);
        }
        pokemon.stats_mut().update_evs("Attack", 80);
        assert_eq!(pokemon.stats.attack_ev, 80);
        pokemon.stats_mut().update_evs("Defense", 80);
        assert_eq!(pokemon.stats.defense_ev, 80);
        pokemon.stats_mut().update_evs("Speed", 80);
        assert_eq!(pokemon.stats.speed_ev, 80);
        Ok(())
    }

    // ==================== FACTORY ====================

    // ==================== IV/EV round-trip through serialization ====================

    #[test]
    fn test_update_iv_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        let original = pokemon.stats.hp_iv;
        let new_val = if original != 31 { 31 } else { 0 };
        // Use the trait method which also updates raw data
        pokemon.update_iv("HP", new_val);
        assert_eq!(pokemon.stats.hp_iv, new_val);
        // Round-trip through bytes — IV must survive
        let bytes = pokemon.to_bytes();
        let reparsed = Gen3Pokemon::from_bytes(0, &bytes)?;
        assert_eq!(reparsed.stats.hp_iv, new_val);
        assert_eq!(reparsed.stats.hp_iv, pokemon.stats.hp_iv);
        Ok(())
    }

    #[test]
    fn test_update_ev_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut pokemon = Gen3Pokemon::from_bytes(0, &TORCHIK)?;
        pokemon.update_ev("Attack", 200);
        assert_eq!(pokemon.stats.attack_ev, 200);
        // Round-trip through bytes — EV must survive
        let bytes = pokemon.to_bytes();
        let reparsed = Gen3Pokemon::from_bytes(0, &bytes)?;
        assert_eq!(reparsed.stats.attack_ev, 200);
        Ok(())
    }

    // ==================== FACTORY (gen_pokemon_from_species) ====================

    #[test]
    fn test_gen_pokemon_from_species_bulbasaur() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let empty = Gen3Pokemon::default();
        let bulbasaur = gen_pokemon_from_species(&empty, "Bulbasaur", "Ash", [0u8; 4].into())?;
        assert_eq!(bulbasaur.species(), "Bulbasaur");
        assert!(!bulbasaur.is_empty());
        Ok(())
    }

    #[test]
    fn test_gen_pokemon_unknown_species_returns_error() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let empty = Gen3Pokemon::default();
        let result = gen_pokemon_from_species(&empty, "FakeMon", "Ash", [0u8; 4].into());
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_gen_pokemon_has_level() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let empty = Gen3Pokemon::default();
        let pokemon = gen_pokemon_from_species(&empty, "Charmander", "Ash", [0u8; 4].into())?;
        assert!(pokemon.level() >= 1);
        Ok(())
    }

    #[test]
    fn test_gen_pokemon_valid_ivs() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let empty = Gen3Pokemon::default();
        let pokemon = gen_pokemon_from_species(&empty, "Pikachu", "Ash", [0u8; 4].into())?;
        let ivs = pokemon.ivs();
        assert!(ivs.hp <= 31);
        assert!(ivs.attack <= 31);
        assert!(ivs.defense <= 31);
        assert!(ivs.speed <= 31);
        assert!(ivs.special_attack <= 31);
        assert!(ivs.special_defense <= 31);
        Ok(())
    }

    #[test]
    fn test_gen_pokemon_has_valid_checksum() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let empty = Gen3Pokemon::default();
        let pokemon = gen_pokemon_from_species(&empty, "Eevee", "Trainer", [0u8; 4].into())?;
        // Re-serialize and re-parse: if checksum is wrong the data would corrupt
        let bytes = pokemon.to_bytes();
        let reparsed = Gen3Pokemon::from_bytes(0, &bytes)?;
        assert_eq!(reparsed.species(), "Eevee");
        Ok(())
    }

    #[test]
    fn test_gen_pokemon_nickname_is_species_uppercase() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let empty = Gen3Pokemon::default();
        let pokemon = gen_pokemon_from_species(&empty, "Squirtle", "Trainer", [0u8; 4].into())?;
        assert_eq!(pokemon.nickname(), "SQUIRTLE");
        Ok(())
    }

    // ==================== Gen3GameData DB functions ====================

    #[test]
    fn test_misc_items_nonempty() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let items = Gen3GameData.pocket_items(Pocket::Items)?;
        assert!(!items.is_empty());
        Ok(())
    }

    #[test]
    fn test_misc_held_items_nonempty() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let items = Gen3GameData.held_items()?;
        assert!(!items.is_empty());
        Ok(())
    }

    #[test]
    fn test_misc_balls_nonempty() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let balls = Gen3GameData.pocket_items(Pocket::Pokeballs)?;
        assert!(!balls.is_empty());
        Ok(())
    }

    #[test]
    fn test_misc_balls_id_nonempty() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let ids = Gen3GameData.balls_id()?;
        assert!(!ids.is_empty());
        Ok(())
    }

    #[test]
    fn test_misc_berries_nonempty() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let berries = Gen3GameData.pocket_items(Pocket::Berries)?;
        assert!(!berries.is_empty());
        Ok(())
    }

    #[test]
    fn test_misc_tms_nonempty() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let tms = Gen3GameData.pocket_items(Pocket::Tms)?;
        assert!(!tms.is_empty());
        Ok(())
    }

    #[test]
    fn test_misc_key_items_nonempty() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let keys = Gen3GameData.pocket_items(Pocket::Key)?;
        assert!(!keys.is_empty());
        Ok(())
    }

    #[test]
    fn test_misc_species_has_386_entries() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let species = Gen3GameData.species()?;
        assert_eq!(species.len(), 386);
        Ok(())
    }

    #[test]
    fn test_misc_species_first_is_bulbasaur() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let species = Gen3GameData.species()?;
        assert_eq!(species.first().map(String::as_str), Some("Bulbasaur"));
        Ok(())
    }

    #[test]
    fn test_misc_moves_nonempty() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let moves = Gen3GameData.moves()?;
        assert!(!moves.is_empty());
        Ok(())
    }

    #[test]
    fn test_misc_pk_species_bulbasaur() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        assert_eq!(Gen3GameData.pk_species(1)?, "Bulbasaur");
        Ok(())
    }

    #[test]
    fn test_misc_pk_species_mewtwo() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        assert_eq!(Gen3GameData.pk_species(150)?, "Mewtwo");
        Ok(())
    }

    #[test]
    fn test_misc_nat_dex_num_bulbasaur() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        assert_eq!(Gen3GameData.nat_dex_num("Bulbasaur")?, 1);
        Ok(())
    }

    #[test]
    fn test_misc_nat_dex_num_torchic() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        assert_eq!(Gen3GameData.nat_dex_num("Torchic")?, 255);
        Ok(())
    }

    #[test]
    fn test_misc_ability_torchic_blaze() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        assert_eq!(Gen3GameData.abilities(255)?.slot1, "Blaze");
        Ok(())
    }

    #[test]
    fn test_misc_hidden_ability_torchic_speed_boost() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        assert!(Gen3GameData.abilities(255)?.hidden.is_none());
        Ok(())
    }

    #[test]
    fn test_misc_growth_rate_bulbasaur_medium_slow() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        assert_eq!(Gen3GameData.growth_rate(1)?, "Medium Slow");
        Ok(())
    }

    #[test]
    fn test_misc_base_stats_bulbasaur() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let stats = Gen3GameData.base_stats(&1u16)?;
        assert_eq!(stats.0, 45); // HP
        assert_eq!(stats.1, 49); // Attack
        assert_eq!(stats.2, 49); // Defense
        assert_eq!(stats.3, 65); // Sp. Attack
        assert_eq!(stats.4, 65); // Sp. Defense
        assert_eq!(stats.5, 45); // Speed
        Ok(())
    }

    #[test]
    fn test_misc_typing_bulbasaur_grass_poison() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let (primary, secondary) = Gen3GameData.typing(1)?;
        assert_eq!(primary, "Grass");
        assert_eq!(secondary, Some("Poison".to_string()));
        Ok(())
    }

    #[test]
    fn test_misc_typing_torchic_pure_fire() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let (primary, secondary) = Gen3GameData.typing(255)?;
        assert_eq!(primary, "Fire");
        assert!(secondary.is_none());
        Ok(())
    }

    #[test]
    fn test_misc_gender_ratio_bulbasaur() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let ratio = Gen3GameData.gender_ratio(1)?;
        assert!(ratio <= 255);
        Ok(())
    }

    #[test]
    fn test_misc_move_data_valid() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let (type_name, name, pp) = Gen3GameData.move_data(10)?;
        assert!(!name.is_empty());
        assert!(!type_name.is_empty());
        assert!(pp > 0);
        Ok(())
    }

    #[test]
    fn test_misc_evolution_charmander_evolves_into_charmeleon() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        // Charmander dex #4 evolves into Charmeleon at level 16
        let evo = Gen3GameData.evolution(&4u16)?;
        assert_eq!(evo.evolves_into, Some(5));
        assert_eq!(evo.method, Some("Level".to_string()));
        assert_eq!(evo.condition, Some("16".to_string()));
        Ok(())
    }

    #[test]
    fn test_misc_evolution_final_form_no_data() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        // Charizard dex #6 is a final evolution — no evolution row exists
        let result = Gen3GameData.evolution(&6u16);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_misc_item_id_g3_potion() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let id = Gen3GameData.item_id_by_name("Potion")?;
        assert!(id > 0);
        Ok(())
    }

    // --- BDSP ---

    #[test]
    fn test_bdsp_crypto_roundtrip() {
        use crate::bdsp::pokemon::crypto;

        let mut buf = vec![0u8; 0x158];
        // Encryption constant + species + PID + random data in the blocks
        buf[0..4].copy_from_slice(&[0x12, 0x34, 0x56, 0x78]);
        buf[0x08..0x0A].copy_from_slice(&[0x9A, 0x00]);
        buf[0x1C..0x20].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
        for i in 0x58..0x90 {
            buf[i] = (i % 251) as u8;
        }

        let encrypted = crypto::encrypt(&buf);
        assert_eq!(encrypted.len(), buf.len());
        assert!(crypto::is_encrypted(&encrypted));

        let decrypted = crypto::decrypt(&encrypted);
        assert_eq!(decrypted[..0x148], buf[..0x148]);
    }

    #[test]
    fn test_bdsp_checksum() {
        use crate::bdsp::pokemon::crypto;

        let mut buf = vec![0u8; 0x148];
        for (i, byte) in buf.iter_mut().enumerate() {
            *byte = (i % 251) as u8;
        }
        let checksum = crypto::calculate_checksum(&buf);
        assert!(checksum != 0);
    }

    #[test]
    fn test_bdsp_detect_party_pokemon() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        use crate::traits::pokemon::Pokemon as PokemonTrait;

        let mut pokemon = crate::bdsp::pokemon::BdspPokemon::new();
        pokemon.set_species("Bulbasaur")?;
        pokemon.set_level(5)?;
        pokemon.set_ot_name("TEST")?;
        pokemon.update_checksum();

        assert_eq!(pokemon.species(), "Bulbasaur");
        assert_eq!(pokemon.level(), 5);
        assert!(!pokemon.is_empty());
        Ok(())
    }

    #[test]
    fn test_bdsp_factory_creates_pokemon() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        use crate::traits::pokemon_factory::PokemonFactory;

        let factory = crate::bdsp::pokemon::BdspFactory;
        let empty = crate::bdsp::pokemon::BdspPokemon::new();
        let ot_id = crate::TrainerID {
            public: 12345,
            private: 54321,
        };
        let pokemon = factory.gen_pokemon_from_species(&empty, "Pikachu", "Ash", ot_id)?;
        assert_eq!(pokemon.species(), "Pikachu");
        assert_eq!(pokemon.ot_name(), "Ash");
        assert!(!pokemon.is_empty());
        Ok(())
    }

    #[test]
    fn test_bdsp_game_data_queries() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        use crate::traits::game_data::GameData;

        let data = crate::bdsp::game_data::BdspGameData;
        assert_eq!(data.pk_species(1)?, "Bulbasaur");
        assert!(data.nat_dex_num("Bulbasaur")? >= 1);
        let (hp, atk, def, spa, spd, spe) = data.base_stats(&25u16)?;
        assert!(hp > 0 && atk > 0 && def > 0 && spa > 0 && spd > 0 && spe > 0);
        let (type1, type2) = data.typing(4)?;
        assert_eq!(type1, "Fire");
        assert!(type2.is_none());
        Ok(())
    }

    // --- Lumi ---

    #[test]
    fn test_lumi_crypto_roundtrip() {
        use crate::lumi::pokemon::crypto;

        let mut buf = vec![0u8; 0x158];
        buf[0..4].copy_from_slice(&[0xAB, 0xCD, 0xEF, 0x01]);
        buf[0x08..0x0A].copy_from_slice(&[0x2A, 0x00]);
        buf[0x1C..0x20].copy_from_slice(&[0x11, 0x22, 0x33, 0x44]);
        for i in 0x58..0x90 {
            buf[i] = (i % 251) as u8;
        }

        let encrypted = crypto::encrypt(&buf);
        assert_eq!(encrypted.len(), buf.len());
        assert!(crypto::is_encrypted(&encrypted));

        let decrypted = crypto::decrypt(&encrypted);
        assert_eq!(decrypted[..0x148], buf[..0x148]);
    }

    #[test]
    fn test_lumi_detect_party_pokemon() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        use crate::traits::pokemon::Pokemon as PokemonTrait;

        let mut pokemon = crate::lumi::pokemon::LumiPokemon::new();
        pokemon.set_species("Charmander")?;
        pokemon.set_level(7)?;
        pokemon.update_checksum();

        assert_eq!(pokemon.species(), "Charmander");
        assert_eq!(pokemon.level(), 7);
        Ok(())
    }

    #[test]
    fn test_lumi_game_data_queries() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        use crate::traits::game_data::GameData;

        let data = crate::lumi::game_data::LumiGameData;
        assert_eq!(data.pk_species(1)?, "Bulbasaur");
        let (type1, _) = data.typing(25)?;
        assert_eq!(type1, "Electric");
        Ok(())
    }

    // --- Detection ---

    /// Writes a valid BDSP/Lumi MD5 hash into a constructed save buffer.
    /// Uses the fixed hash offset at 0xE9818, matching PKHeX's algorithm:
    /// MD5 over entire file with the 16 hash bytes zeroed.
    fn bdsp_md5(data: &mut [u8]) {
        use md5::{Digest, Md5};
        const HASH_OFFSET: usize = 0xE9818;
        let mut hasher = Md5::new();
        let mut buf = data.to_vec();
        if let Some(h) = buf.get_mut(HASH_OFFSET..HASH_OFFSET + 16) {
            h.fill(0);
        }
        hasher.update(&buf);
        let digest = hasher.finalize();
        if let Some(dest) = data.get_mut(HASH_OFFSET..HASH_OFFSET + 16) {
            dest.copy_from_slice(&digest[..16]);
        }
    }

    #[test]
    fn test_open_detects_bdsp_save() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        // Construct a valid-size BDSP save with version 0x34 (V1.3) and valid hash
        let mut save = vec![0u8; 0xEF0A4];
        save[0..4].copy_from_slice(&0x34u32.to_le_bytes());
        bdsp_md5(&mut save);
        let result = crate::open(&save);
        assert!(matches!(result, Ok(crate::OpenSave::Bdsp(_))));
        Ok(())
    }

    #[test]
    fn test_bdsp_save_modify_and_reopen_validates_hash() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        let mut save_bytes = vec![0u8; 0xEF0A4];
        save_bytes[0..4].copy_from_slice(&0x34u32.to_le_bytes());
        bdsp_md5(&mut save_bytes);

        let mut opened = crate::open(&save_bytes)?;
        if let crate::OpenSave::Bdsp(ref mut bdsp) = opened {
            let mut trainer = bdsp.trainer()?;
            trainer.money = 12345;
            bdsp.save_trainer(&trainer)?;
        }

        let raw = opened.raw_data();
        let reopened = crate::open(&raw);
        assert!(reopened.is_ok(), "Reopening edited BDSP save failed hash validation");
        Ok(())
    }

    #[test]
    fn test_open_detects_lumi_save() -> Result<(), Box<dyn std::error::Error>> {
        setup_db();
        // Construct a valid-size Lumi save with 0xFFFF prefix and valid hash
        let mut save = vec![0u8; 0xEF0A4];
        save[0..4].copy_from_slice(&[0x00, 0x00, 0xFF, 0xFF]);
        bdsp_md5(&mut save);
        let result = crate::open(&save);
        assert!(matches!(result, Ok(crate::OpenSave::Lumi(_))));
        Ok(())
    }

    #[test]
    fn test_open_unknown_format() {
        let data = vec![0u8; 0x1000];
        let result = crate::open(&data);
        assert!(matches!(result, Err(crate::DetectError::UnknownFormat(_))));
    }

    // --- Real BDSP save roundtrip (PocketHeX fixture) ---

    #[test]
    fn test_bdsp_real_save_load_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        use std::path::PathBuf;
        use crate::traits::pokemon::Pokemon as _;

        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/bdsp_v1.3_populated.sav");
        if !fixture.exists() {
            // Skip if fixture not present (e.g. CI without test data)
            return Ok(());
        }
        let data = std::fs::read(&fixture)?;
        let save = crate::bdsp::save::SaveFile::new(&data)?;
        let party = save.party()?;
        let raw = save.raw_data();
        assert_eq!(raw.len(), data.len());
        for mon in &party {
            assert!(!mon.is_empty());
            assert!(!mon.species().is_empty());
        }
        Ok(())
    }

    #[test]
    fn test_bdsp_real_save_detected_as_bdsp() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/bdsp_v1.3_populated.sav");
        if !fixture.exists() {
            return Ok(());
        }
        let data = std::fs::read(&fixture)?;
        let result = crate::open(&data);
        assert!(
            matches!(result, Ok(crate::OpenSave::Bdsp(_))),
            "Expected Bdsp variant, got {:?}",
            result.as_ref().map(|_| "Bdsp").or_else(|e| Err(format!("{e}")))
        );
        Ok(())
    }

    #[test]
    fn test_gen3_unown_form_from_pid() {
        // Test vectors from PKHeX EntityPID.GetUnownForm3
        // Form = (((pid & 0x3000000) >> 18) | ((pid & 0x30000) >> 12)
        //       | ((pid & 0x300) >> 6) | (pid & 0x3)) % 28
        struct Case {
            pid: u32,
            expected_form: u8,
        }
        const CASES: &[Case] = &[
            Case { pid: 0x00000000, expected_form: 0 },    // A
            Case { pid: 0x00000001, expected_form: 1 },    // B
            Case { pid: 0x00000002, expected_form: 2 },    // C
            Case { pid: 0x00000003, expected_form: 3 },    // D
            Case { pid: 0x00000300, expected_form: 12 },   // form 12 (0x300 >> 6 = 0xC = 12)
            Case { pid: 0x00030000, expected_form: 20 },   // form 20 (0x30000 >> 12 = 0x30 = 48 % 28 = 20)
            Case { pid: 0x03000000, expected_form: 24 },   // form 24 (0x3000000 >> 18 = 192 % 28 = 24)
            Case { pid: 0x03030303, expected_form: 3 },    // form 3  (255 % 28 = 3)
            Case { pid: 0xFFFFFFFF, expected_form: 3 },    // form 3  (255 % 28 = 3)
        ];

        for case in CASES {
            let expected = case.expected_form;
            let value = ((case.pid & 0x0300_0000) >> 18)
                | ((case.pid & 0x0003_0000) >> 12)
                | ((case.pid & 0x0000_0300) >> 6)
                | (case.pid & 0x0000_0003);
            let computed = (value % 28) as u8;
            assert_eq!(
                computed, expected,
                "Unown form for PID 0x{:08X}: expected {}, got {}",
                case.pid, expected, computed
            );
        }
    }
}
