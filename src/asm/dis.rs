use super::IType;

impl IType{
    /// Function word
    pub fn f(&self) -> Option<u32>{
        match self{
            IType::R(v) => {
                let mut sum = 0;
                sum += (v >> 12) & 0b111;
                sum += ( (v >> 25) & 0b111_1111 ) << 3;
                Some(sum)
            },
            IType::I(v) => {
                Some( (v >> 12) & 0b111 )
            },
            IType::S(v) => {
                Some( (v >> 12) & 0b111 )
            },
            IType::U(v) => {
                None
            }
        }
    }
    /// Source 1 register
    pub fn s1(&self) -> Option<u32>{
        match self{
            IType::U(v) => {
                None
            },
            IType::R(v) | IType::I(v) | IType::S(v) => {
                Some( (v >> 15) & 0x1F )
            }
        }
    }
    /// Source 2 register
    pub fn s2(&self) -> Option<u32>{
        match self{
            IType::R(v) | IType::S(v) => {
                Some( (v >> 20) & 0x1F )
            },
            _ => {
                None
            }
        }
    }
    /// immediate
    pub fn imm(&self) -> Option<u32>{
        match self{
            IType::I(v) => {
                Some( (v >> 20) & 0xFFF )
            },
            IType::S(v) => {
                let mut sum = 0;
                sum += (v >> 7) & 0x1F;
                sum += ((v >> 25) & 0b111_1111) << 5;
                Some(sum)
            },
            _ => {
                None
            }
        }
    }
    /// opcode
    pub fn opcode(&self) -> u32{
        match self{
            IType::I(v) | IType::S(v) | IType::R(v) | IType::U(v) => {
                v & 0b111_1111
            }
        }
    }
}