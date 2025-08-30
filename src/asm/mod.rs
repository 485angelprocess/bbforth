/******************/
/** RISC-V CPU ****/
/******************/
mod dis;

enum IType{
    R(u32),
    I(u32),
    S(u32),
    U(u32)    
}

impl IType{
    /// R-Type
    fn build_rtype(fu: u32, fl: u32, s1: u32, s2: u32, d: u32, opcode: u32) -> Self{
        let mut sum: u32 = 0;
        sum += (fu & 0b111_1111) << 25;
        sum += (fl & 0b111) << 12;
        sum += (s1 & 0b11111) << 15;
        sum += (s2 & 0b11111) << 20;
        sum += (d & 0b11111) << 7;
        sum += opcode & 0b111_1111;
        IType::R(sum)
    }
    
    /// I Type
    fn build_itype(f: u32, imm: u32, s1: u32, d: u32, opcode: u32) -> Self{
        let mut sum: u32 = 0;
        sum += (f & 0b111) << 12;
        sum += (imm & 0xFFF) << 20;
        sum += (s1 & 0b11111) << 15;
        sum += (d & 0b11111) << 7;
        sum += opcode & 0b111_1111;
        IType::I(sum)
    }
    
    /// S Type
    fn build_stype(f: u32, imm: u32, s1: u32, s2: u32, opcode: u32) -> Self{
        let mut sum: u32 = 0;
        sum += (f & 0b111) << 12;
        sum += ((imm >> 5) & 0b11_1111) << 25;
        sum += (imm & 0b11111) << 7;
        sum += (s1 & 0b11111) << 15;
        sum += (s2 & 0b11111) << 20;
        sum += opcode & 0b111_1111;
        IType::S(sum)
    }
    
    /// U Type
    fn build_utype(imm: u32, d: u32, opcode: u32) -> Self{
        let mut sum: u32 = 0;
        sum += ((imm >> 12) & 0xFFFFF) << 12;
        sum += (d & 0b11111) << 7;
        sum += opcode & 0b111_1111;
        IType::U(sum)
    }
}

// TODO: get all the named ones
const NamedRegs: [&str; 2] = [
    "zero",
    "sp"
];

enum Argument{
    Register(u8),
    Imm(u32),
    Label(String, Option<u32>)
}

impl Argument{
    
}

enum Assemble{
    Label(String),
    Un(String, Argument, Argument),
    Bin(String, Argument, Argument, Argument),
    Offset(String, Argument, Argument, Argument)
}

impl Assemble{
    
}