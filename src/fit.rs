pub struct Fit {}

impl Fit {
    fn crc_get16(crc: u16, byte: u8) -> u16 {
        static CRC_TABLE: [u16; 16] = [
            0x0000, 0xCC01, 0xD801, 0x1400, 0xF001, 0x3C00, 0x2800, 0xE401, 0xA001, 0x6C00, 0x7800,
            0xB401, 0x5000, 0x9C01, 0x8801, 0x4400,
        ];
        let byte = usize::from(byte);
        let (low, high) = (byte & 0xF, (byte >> 4) & 0xF);

        let crc_0 = (crc >> 4) & 0x0FFF;
        let crc_1 = crc_0 ^ CRC_TABLE[usize::from(crc & 0xF)] ^ CRC_TABLE[low];
        let crc_2 = (crc_1 >> 4) & 0x0FFF;

        crc_2 ^ CRC_TABLE[usize::from(crc_1 & 0xF)] ^ CRC_TABLE[high]
    }

    pub fn crc_calc16(data: &[u8]) -> u16 {
        data.iter()
            .fold(0, |crc, datum| Self::crc_get16(crc, *datum))
    }
}

#[test]
fn test_crc_calc16() {
    // Nothing special about 0xFED, other than the fact that I've been playing
    // with nom.
    static FED: [u8; 3] = [0xF, 0xE, 0xD];

    assert_eq!(0, Fit::crc_calc16(b""));
    // NOTE: I compiled the C code from the Fit SDK and tested these slices.
    assert_eq!(0xE0F0, Fit::crc_calc16(b"This is a test"));
    assert_eq!(0x0440, Fit::crc_calc16(&FED[..1]));
    assert_eq!(0x3484, Fit::crc_calc16(&FED[..2]));
    assert_eq!(0xA6F5, Fit::crc_calc16(&FED));
}
