fn fit_crc_get16(mut crc: u16, byte: u8) -> u16 {
    static CRC_TABLE: [u16; 16] = [
      0x0000, 0xCC01, 0xD801, 0x1400, 0xF001, 0x3C00, 0x2800, 0xE401,
      0xA001, 0x6C00, 0x7800, 0xB401, 0x5000, 0x9C01, 0x8801, 0x4400
    ];
    let mut tmp = CRC_TABLE[usize::from(crc & 0xf)];
    crc = (crc >> 4) & 0x0fff;
    crc = crc ^ tmp ^ CRC_TABLE[usize::from(byte & 0xf)];

    tmp = CRC_TABLE[usize::from(crc & 0xf)];
    crc = (crc >> 4) & 0x0fff;
    crc =  crc ^ tmp ^ CRC_TABLE[usize::from((byte >> 4) & 0xf)];
    
    crc
}

pub fn fit_crc_calc16(data: &[u8]) -> u16 {
    data.iter().fold(0, |crc, datum| fit_crc_get16(crc, *datum))
}

#[test]
fn test_fit_crc_calc16() {
    static FED: [u8; 3] = [0xf, 0xe, 0xd];

    assert_eq!(0, fit_crc_calc16(b""));
    // NOTE: I compiled the C code from the Fit SDK and tested these slices.
    assert_eq!(0xE0F0, fit_crc_calc16(b"This is a test"));
    assert_eq!(0x0440, fit_crc_calc16(&FED[..1]));
    assert_eq!(0x3484, fit_crc_calc16(&FED[..2]));
    assert_eq!(0xA6F5, fit_crc_calc16(&FED));
}
