use std::collections::HashMap;

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

type LabelLookup = HashMap<String, u32>;

enum Argument{
    Register(u8),
    Imm(u32),
    Label(String, Option<u32>)
}

impl Argument{
    /// Set address of label, returns true if the label was set
    fn set(&mut self, labels: &LabelLookup) -> bool{
        match self{
            Argument::Label(l, v) => {
                if let Some(addr) = labels.get(l){
                    *v = Some(*addr);
                    return true;
                }
            }
            _ => ()
        };
        false
    }
    
    fn label_undefined(&self) -> bool{
        match self{
            Argument::Label(_l, v) =>{
                return v.is_none();
            },
            _ => ()
        }
        return true;
    }
}

enum Assemble{
    Label(String),
    Statement(String),
    Un(String, Argument, Argument),
    Bin(String, Argument, Argument, Argument),
    Offset(String, Argument, Argument, Argument)
}

enum IFormat{
    Statement,
    Unary,
    Binary,
    Offset
}

enum IParam{
    R(u32, u32),
    IA(u32, u32), //immediate arithmetic
    I(u32),
    U, // u type like lui or auipc
    B(u32), // branch
    S(u32), // Store
    JALR,
    JAL,
    None
}

const FADD: u32 = 0x0;
const FSHL: u32 = 0x1;
const FSLT: u32 = 0x2; // set less than
const FSLU: u32 = 0x3; // set less than unsigned
const FXOR: u32 = 0x4;
const FSHR: u32 = 0x5;
const FOR:  u32 = 0x6;
const FAND: u32 = 0x7;

const OPREGARITH: u32 = 0b0110011;
const OPIMMARITH: u32 = 0b0010011;
const OPLOAD: u32     = 0b0000011;
const OPBRANCH: u32   = 0b1100011;
const OPSTORE: u32    = 0b0100011;

struct InstructionBuilder{
    opcode: u32,
    format: IFormat,
    param: IParam
}

impl InstructionBuilder{
    fn new(opcode: u32, format: IFormat, param: IParam) -> Self{
        Self{
            opcode: opcode,
            format: format,
            param: param
        }
    }
    
    fn binary(opcode: u32, param: IParam) -> Self{
        Self::new(opcode, IFormat::Binary, param)
    }
    
    fn reg_arith(f7: u32, f3: u32) -> Self{
        Self::new(OPREGARITH, IFormat::Binary, IParam::R(f7, f3))
    }
    
    fn imm_arith(f7: u32, f3: u32) -> Self{
        Self::new(OPIMMARITH, IFormat::Binary,
            IParam::IA(f7, f3))
    }
    
    fn load(f3: u32) -> Self{
        Self::new(OPLOAD, IFormat::Offset, IParam::I(f3))
    }
    
    fn store(f3: u32) -> Self{
        Self::new(OPSTORE, IFormat::Offset, IParam::S(f3))
    }
    
    fn utype(opcode: u32) -> Self{
        Self::new(opcode, IFormat::Unary, IParam::U)
    }
    
    fn branch(f3: u32) -> Self{
        Self::new(OPBRANCH, IFormat::Binary, IParam::B(f3))
    }
}

#[derive(Clone)]
struct LineReader{
    tokens: Vec<String>,
    position: usize
}

impl LineReader{
    fn new(t: Vec<String>) -> Self{
        Self{
            tokens: t,
            position: 0
        }
    }
    
    fn next(&mut self) -> Option<&String>{
        let t = self.tokens.get(self.position);
        self.position += 1;
        t
    }
    
    fn insert(&mut self, v: &String){
        self.tokens.insert(0, v.clone());
    }
}

pub struct Assembler{
    labels: LabelLookup,
    // TODO variables
    opdict: HashMap<&'static str, InstructionBuilder>,
    line_number: u32,
    alias: HashMap<&'static str, Vec<String>>,
    pub file_number: u32
}

impl Assembler{
    pub fn new() -> Self{
        let mut asm = Self{
            labels: LabelLookup::new(),
            opdict: HashMap::new(),
            alias: HashMap::new(),
            line_number: 0,
            file_number: 0
        };
        asm.setup();
        asm
    }
    
    fn insert(&mut self, inst: &'static str, ib: InstructionBuilder){
        self.opdict.insert(inst, ib);
    }
    
    fn setup(&mut self){
        // Immediate arithmetic
        self.insert("addi", InstructionBuilder::imm_arith(0, FADD));
        self.insert("xori", InstructionBuilder::imm_arith(0, FXOR));
        self.insert("ori", InstructionBuilder::imm_arith(0, FOR));
        self.insert("andi", InstructionBuilder::imm_arith(0, FAND));
        self.insert("slli", InstructionBuilder::imm_arith(0, FSHL));
        self.insert("srli", InstructionBuilder::imm_arith(0, FSHR));
        self.insert("srai", InstructionBuilder::imm_arith(0x20, FSHR));
        self.insert("slti", InstructionBuilder::imm_arith(0x20, FSLT));
        self.insert("sltiu", InstructionBuilder::imm_arith(0x20, FSLU));
        
        // Register functions
        self.insert("add", 
            InstructionBuilder::reg_arith(0, FADD));
        self.insert("sub", 
            InstructionBuilder::reg_arith(0x20, FADD));
        self.insert("xor", 
            InstructionBuilder::reg_arith(0, FXOR));
        self.insert("or", 
            InstructionBuilder::reg_arith(0, FOR));
        self.insert("and", 
            InstructionBuilder::reg_arith(0, FAND));
        self.insert("sll", 
            InstructionBuilder::reg_arith(0, FSHL));
        self.insert("srl", 
            InstructionBuilder::reg_arith(0, FSHR));
        self.insert("sra", 
            InstructionBuilder::reg_arith(0x20, FSHR));
        self.insert("slt", 
            InstructionBuilder::reg_arith(0, FSLT));
        self.insert("sltu", 
            InstructionBuilder::reg_arith(0, FSLU));
            
        // Load
        self.insert("lb", InstructionBuilder::load(0b000));
        self.insert("lh", InstructionBuilder::load(0b001));
        self.insert("lw", InstructionBuilder::load(0b010));
        self.insert("lbu", InstructionBuilder::load(0b100));
        self.insert("lhu", InstructionBuilder::load(0b101));
            
        // Store
        self.insert("sb", InstructionBuilder::store(0b000));
        self.insert("sh", InstructionBuilder::store(0b001));
        self.insert("sw", InstructionBuilder::store(0b010));
        
            
        // Branch
        self.insert("beq", InstructionBuilder::branch(0x0));
        self.insert("bne", InstructionBuilder::branch(0x1));
        self.insert("blt", InstructionBuilder::branch(0x4));
        self.insert("bge", InstructionBuilder::branch(0x5));
        self.insert("bltu", InstructionBuilder::branch(0x6));
        self.insert("bgeu", InstructionBuilder::branch(0x7));
    
        // Immediate
        self.insert("lui", InstructionBuilder::utype(0b0110111));
        self.insert("auipc", InstructionBuilder::utype(0b0010111));
        
        self.insert("ecall", InstructionBuilder::new(
            0b1110011,
            IFormat::Statement,
            IParam::None
        ));
        
        self.insert("jal", InstructionBuilder::new(
            0b1101111,
            IFormat::Binary,
            IParam::JAL
        ));
        
        self.insert("jalr", InstructionBuilder::new(
            0b1100111,
            IFormat::Binary,
            IParam::JALR
        ));
        
        self.alias.insert("j", vec![
            "jal".to_string(),
            "x0".to_string()
        ]);
    }
    
    fn read_instruction(&mut self, inst: &String, reader: &mut LineReader, _comment: Option<String>) -> Option<Assemble>{
        if let Some(builder) = &self.opdict.get(inst.as_str()){
            
        }
        else if let Some(alias) = &self.alias.get(inst.as_str()){
            let mut malias: Vec<String> = alias.to_vec();
            malias.reverse();
            for a in malias{
                reader.insert(&a);
            }
        }
        else{
            panic!("Line {}: Instruction {} not defined", self.file_number, inst);
        }
        None   
    }
    
    fn read_tokens(&mut self, reader: &mut LineReader, comment: Option<String>) -> Vec<Assemble>{
        let mut assembled = Vec::new();
        if let Some(token) = reader.next(){
            if token.ends_with(":"){
                let label = token[0..token.len()-1].to_string();
                println!("Found label {}: {}", label, self.line_number);
                self.labels.insert(label.clone(), self.line_number);
                assembled.push(Assemble::Label(label))
                
                // todo find any unopened labels
            }
            else{
                self.read_instruction(&token.clone(), reader, comment);
                self.line_number += 1;
            }
        }
        assembled
    }
    
    pub fn read_line(&mut self, line: &String) -> usize{
        let lp = line.trim().to_string();
        
        // Remove comments
        if lp.starts_with("#") || lp.is_empty(){
            self.file_number += 1;
            return 0;
        }
        
        let comment_split: Vec<&str> = lp.split("#").collect();
        
        let mut comment = None;
        
        if comment_split.len() == 0{
            self.file_number += 1;
            return 0;
        }
        
        if comment_split.len() > 1{
            // edge case that the comment has # but whatever
            comment = Some(comment_split[1].to_string());
        }
        
        let lnc = comment_split[0];
        
        // Split into tokens
        let tokens: Vec<String> = lnc.split(&[' ', ','][..]).map(|t|(t.to_string())).collect();
        
        let mut reader = LineReader::new(tokens);
        
        self.read_tokens(&mut reader, comment);
        
        self.file_number += 1;
        1
    }
}