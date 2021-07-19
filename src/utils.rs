pub fn number_to_hex(number: u8) -> char {
    char::from_digit(number as u32, 16)
        .unwrap()
        .to_ascii_uppercase()
}
