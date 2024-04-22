#[cfg(test)]
mod tests {
    use crate::data_structure::pokemon::{Pokemon, Pokerus};

    const TORCHIK: [u8; 100] = [
        101, 231, 167, 198, 154, 166, 220, 6, 206, 201, 204, 189, 194, 195, 189, 255, 1, 0, 2, 2,
        195, 213, 226, 255, 255, 255, 255, 0, 49, 30, 0, 0, 255, 65, 123, 193, 255, 65, 123, 192,
        255, 65, 123, 192, 231, 64, 123, 192, 103, 65, 123, 192, 255, 7, 123, 192, 255, 81, 254,
        225, 69, 32, 147, 217, 255, 65, 123, 192, 245, 65, 86, 192, 255, 65, 123, 192, 220, 105,
        123, 192, 0, 0, 0, 0, 5, 255, 20, 0, 20, 0, 11, 0, 10, 0, 9, 0, 14, 0, 10, 0,
    ];

    #[test]
    fn ability() {
        let torchik = Pokemon::new(0, &TORCHIK);
        let ability = torchik.ability();
        assert_eq!("Blaze", ability);
    }

    #[test]
    fn is_egg() {
        let torchik = Pokemon::new(0, &TORCHIK);
        let is_egg = torchik.is_egg();
        assert_eq!(false, is_egg);
    }

    #[test]
    fn moves() {
        let torchik = Pokemon::new(0, &TORCHIK);
        let moves: Vec<(String, String, u8, u8)> = vec![
            (
                "Normal".to_string(),
                "Scratch".to_string(),
                35 as u8,
                35 as u8,
            ),
            (
                "Normal".to_string(),
                "Growl".to_string(),
                40 as u8,
                40 as u8,
            ),
        ];
        assert_eq!(moves, torchik.moves());
    }

    #[test]
    fn pokerus() {
        let torchik = Pokemon::new(0, &TORCHIK);

        assert_eq!(Pokerus::None, torchik.pokerus_status());
    }

    #[test]
    fn stats() {
        let torchik = Pokemon::new(0, &TORCHIK);

        torchik.stats();

        assert_eq!(true, true);
    }
}
