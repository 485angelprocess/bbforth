use crate::reg;
use crate::types::ForthVal;

const JUMP: u32 = 0;
// Core data structure
const NEXT: u32 = 4;
const STACKBASE: u32 = 8; // Start of stack pointer
// Local data
// Local program

const STACKSIZE: u32 = 128;

mod R{
    pub const zero: u32 = 0x0;
    pub const sp: u32   = 0x1;
    pub const t1: u32   = 0x6;
}

#[derive(Clone)]
pub struct Instruction{
    op: u32
}

impl Instruction{
    fn jal(rd: u32, imm: u32) -> Self{
        let opcode = 0b1101111;
        let mut i = 0;
        i += opcode;
        i += (rd & 0x1F) << 7;
        i += ((imm >> 12) & 0b11111111) << 12;
        i += ((imm >> 11) & 0b1) << 20;
        i += ((imm >> 1) & 0b1111111111) << 21;
        i += ((imm >> 20) & 0b1) << 31;
        Self{op: i}
    }
}

#[derive(Clone)]
pub struct Proc{
    values: Vec<(String, ForthVal)>,
    main: Vec<Instruction>
}

impl Default for Proc{
    fn default() -> Self {
        Self::new()
    }
}

impl Proc{
    pub fn new() -> Self{
        Self{
            values: vec![("_next".to_string(), ForthVal::Int(-1))],
            main: Vec::new()
        }
    }
    
    pub fn append_program_as_u32(&mut self, i: u32){
        self.main.push(Instruction{op: i});
    }
    
    pub fn append_program(&mut self, i: Instruction){
        self.main.push(i);
    }
    
    pub fn adjust_stack(&mut self){
        todo!("Insert stack adjustment routine");
    }
    
    pub fn go_to_next(&mut self){
        todo!("Insert go to next instruction");
    }
    
    pub fn program_start_offset(&self) -> u32{
        let mut sum = 8; // initial size
        sum += self.local_size();
        sum
    }
    
    pub fn local_size(&self) -> u32{
        let mut sum = 0;
        for (_name, v) in &self.values{
            sum += v.size() as u32;
        }
        sum
    }
    
    pub fn program_size(&self) -> u32{
        // tODO should i check for pseudo instructions here? probably not
        self.main.len() as u32
    }
    
    pub fn load_start(&self) -> Instruction{
        println!("Local size is {}", self.local_size());
        Instruction::jal(R::t1, (self.local_size()+1) << 2)
    }
}