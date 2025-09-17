fn bitmap(v: u32, mask: u32, base: u32, shift: u32) -> u32{
    ((v >> base) & mask) << shift
}

pub fn jal(rd: u32, offset: u32) -> u32{
    let offsetmap = bitmap(offset, 0xFF, 12, 0) +
                    bitmap(offset, 0x1, 11, 8) +
                    bitmap(offset, 0x3FF, 1, 9) +
                    bitmap(offset, 0x1, 20, 19);
    let opcode = 0b1101111;
    return 
        opcode +
        (rd << 7) +
        (offsetmap << 12)
    ;
}