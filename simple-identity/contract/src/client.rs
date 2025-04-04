pub mod metadata {
    pub const ELF: &[u8] = include_bytes!("../../methods/guest/guest.img");
    pub const PROGRAM_ID: [u8; 32] = sdk::str_to_u8(include_str!("../../methods/guest/guest.txt"));
}
