use std::{backtrace::BacktraceStatus, borrow::{Borrow, BorrowMut}, collections::HashMap};

type  BaseType = i32;

#[derive(Debug, Clone, Copy)]
pub enum PRIM{
    LIT,
    SYS,
    DUP,
    MUL,
    LTZ,
    EXIT,
    COL,
    SEMICOL,
    SWAP,
    ROT,
    DROP,
    ADD,
    PICK
}

pub enum Mode{
    STANDARD,
    DEFINE,
    COMPILE
}

#[derive(Clone)]
struct Word{
    pub contents: Vec<(usize, BaseType)>
}

impl Word{
    pub fn new(op: usize, v:BaseType) -> Self{
        Self{
            contents: vec![(op, v)]
        }
    }
    pub fn push(&mut self, op: usize, v: BaseType){
        self.contents.push((op, v));
    }
}

pub struct Context{
    data_stack: Vec<BaseType>,
    
    dict: HashMap<String, usize>,
    prim: HashMap<usize, PRIM>,
    words: HashMap<usize, Word>,
    
    active_word: Option<String>,
    active_contents: Option<Word>,
    
    mode: Mode,
    op: usize,
    sys: usize,
    lit: usize,
    pub sys_buffer: Vec<BaseType>
}

impl Context{
    pub fn new() -> Self{
        let mut ctx = Self{
            data_stack: Vec::new(),
            
            dict: HashMap::new(),
            prim: HashMap::new(),
            words: HashMap::new(),
            
            active_word: None,
            active_contents: None,
            
            mode: Mode::STANDARD,
            
            sys_buffer: Vec::new(),
            
            op: 0,
            sys: 0,
            lit: 0
        };
        ctx.setup();
        ctx
    }
    
    pub fn clear_sys_buffer(&mut self){
        self.sys_buffer.clear();
    }
    
    fn add_implicit(&mut self, p: PRIM) -> usize{
        let op = self.op;
        self.prim.insert(self.op, p);
        self.op += 1;
        op
    }
    
    fn add_prim(&mut self, s: &str, p: PRIM){
        self.dict.insert(s.to_string(), self.op);
        self.prim.insert(self.op, p);
        self.op += 1;
    }
    
    pub fn setup(&mut self){
        self.lit = self.add_implicit(PRIM::LIT);
        self.sys = self.add_implicit(PRIM::SYS);
        
        self.add_prim("dup", PRIM::DUP);
        self.add_prim("*", PRIM::MUL);
        self.add_prim("ltz", PRIM::LTZ);
        self.add_prim(":", PRIM::COL);
        self.add_prim(";", PRIM::SEMICOL);
        self.add_prim("swap", PRIM::SWAP);
        self.add_prim("rot", PRIM::ROT);
        self.add_prim("+", PRIM::ADD);
        self.add_prim("drop", PRIM::DROP);
        self.add_prim("pick", PRIM::PICK);
    }
    
    fn start_definition(&mut self){
        self.mode = Mode::DEFINE;
    }
    
    fn end_definition(&mut self){
        // a few steps
        self.mode = Mode::STANDARD;
        
        // register word
        self.dict.insert(self.active_word.as_ref().unwrap().clone(), self.op);
        self.words.insert(self.op, self.active_contents.as_ref().unwrap().clone());  
        
        // clear cache
        self.active_word = None;
        self.active_contents = None;
        
        // Keep track of defined words
        self.op += 1;
    }
    
    fn run_word(&mut self, op: usize){
        let w = &self.words[&op];
        
        // TODO clean up and also prevent infinite nesting
        for (op, v) in w.contents.clone(){
            if self.prim.contains_key(&op){
                self.do_prim(self.prim[&op], v);
            }
            else if self.words.contains_key(&op){
                self.run_word(op);
            }
            else{
                panic!("Operation {} not defined", op);
            }
        }
    }
    
    pub fn parse(&mut self, msg: &String){
        
        match self.mode{
            Mode::DEFINE => {
                self.active_word = Some(msg.clone());
                println!("Adding word {}", self.active_word.as_ref().unwrap());
                self.mode = Mode::COMPILE;
            },
            Mode::COMPILE => {
                let (op, v) = self.get_op(msg);
                
                // this is ugly but im going to leave this here
                if self.prim.contains_key(&op){
                    let p = self.prim[&op];
                    match p{
                        PRIM::SEMICOL => self.end_definition(),
                        _ => {
                            if let Some(c) = &mut self.active_contents{
                                c.push(op, v);
                            }
                            else{
                                self.active_contents = Some(Word::new(op, v));
                            }
                        }
                    }
                }
                
                
            },
            Mode::STANDARD => {
                // get address of word with argument
                let (op, v) = self.get_op(msg);
                
                if self.prim.contains_key(&op){
                    self.do_prim(self.prim[&op], v);
                }
                else if self.words.contains_key(&op){
                    
                    self.run_word(op);
                }
                else{
                    panic!("Operation {} not defined", op);
                }
            }
        }
        
        
    }
            
    fn get_op(&mut self, msg: &String) -> (usize, i32){
        
        if msg.starts_with("."){
            // System call/special functions
            if let Some(result) = msg.bytes().nth(1){
                // system call with specifier
                // These aren't implemented yet,
                // But ." is literal string
                // and .s displays the entire stack as examples
                return (self.sys, result.try_into().unwrap());
            }
            else{
                // system call no argument
                return (self.sys, 0);
            }
        }
        if self.dict.contains_key(msg){
            // in dictionary of words
            return (self.dict[msg], 0);
        }
        // assume it is literal
        (self.lit, msg.parse().unwrap())
    }
    
    fn push(&mut self, v: BaseType){
        // PUSH onto stack
        // TODO: checks
        self.data_stack.push(v);
    }
    fn pop(&mut self) -> BaseType{
        self.data_stack.pop().unwrap()
    }
    
    fn sys_call(&mut self){
        // tODO ids
        let s = self.pop();
        print!("{}", s); // .
        self.sys_buffer.push(s);
    }
    
    pub fn do_prim(&mut self, p: PRIM, v: BaseType){
        // Run primitives
        match p{
            PRIM::LIT => self.push(v),
            PRIM::SYS => self.sys_call(),
            PRIM::COL => self.start_definition(),
            PRIM::SEMICOL => self.end_definition(),
            PRIM::LTZ => {
                if self.pop() < 0{
                    self.push(1);
                }
                else{
                    self.push(0);
                }
            },
            PRIM::DUP => {
                let v = self.pop();
                self.push(v);
                self.push(v);
            },
            PRIM::MUL => {
                let a = self.pop();
                let b = self.pop();
                self.push(a * b);
            },
            PRIM::SWAP => {
                // Swap first two lines on the stack
                let a = self.pop();
                let b = self.pop();
                self.push(a);
                self.push(b);
            },
            PRIM::ROT => {
                let a = self.pop();
                let b = self.pop();
                let c = self.pop();
                    self.push(b);
                    self.push(a);
                    self.push(c);
            }
            PRIM::DROP => {let _ = self.pop();},
            PRIM::ADD => {
                let a = self.pop();
                let b = self.pop();
                self.push(a + b);
            },
            PRIM::PICK => {
              let addr = self.pop();
              self.push(self.data_stack[addr as usize]);  
            },
            _ => todo!("Not implemented {:?}", p)
        }
    }
}

pub fn run_program(program: &str, ctx: &mut Context){
    for p in program.split(" "){
        ctx.parse(&p.to_string());
    }
}

#[cfg(test)]
mod tests{
    use super::{run_program, Context};

    #[test]
    fn dup(){
        let mut ctx = Context::new();
        run_program("5 dup . .", &mut ctx);
        assert!(ctx.sys_buffer.pop().unwrap() == 5);
        assert!(ctx.sys_buffer.pop().unwrap() == 5);
    }
    
    #[test]
    fn literal(){
        let mut ctx = Context::new();
        run_program("1 2 3 . . .", &mut ctx);
        assert!(ctx.sys_buffer[0] == 3);
        assert!(ctx.sys_buffer[1] == 2);
        assert!(ctx.sys_buffer[2] == 1);
    }
    
    #[test]
    #[should_panic]
    fn dup_requires_argument(){
        let mut ctx = Context::new();
        run_program("dup .", &mut ctx);
    }
    
    #[test]
    #[should_panic]
    fn undefined_word(){
        let mut ctx = Context::new();
        run_program("newword", &mut ctx);
    }
    
    #[test]
    fn ltz(){
        let mut ctx = Context::new();
        run_program("-1 ltz .", &mut ctx);
        assert!(ctx.sys_buffer.pop().unwrap() == 1);
        
        run_program("0 ltz .", &mut ctx);
        assert!(ctx.sys_buffer.pop().unwrap() == 0);
        
        run_program("1 ltz .", &mut ctx);
        assert!(ctx.sys_buffer.pop().unwrap() == 0);
    }
    
    #[test]
    fn new_word(){
        let mut ctx = Context::new();
        
        // new word
        run_program(": square dup * ;", &mut ctx);
        
        // run program with new word
        run_program("5 square .", &mut ctx);
        
        assert!(ctx.sys_buffer.pop().unwrap() == 25);
    }
    
    #[test]
    #[should_panic]
    fn new_word_no_body(){
        let mut ctx = Context::new();
        
        run_program(": square ;", &mut ctx);
    }
    
    #[test]
    fn rot(){
        let mut ctx = Context::new();
        run_program("1 2 3 rot . . .", &mut ctx);
        assert!(ctx.sys_buffer[0] == 1);
        assert!(ctx.sys_buffer[1] == 3);
        assert!(ctx.sys_buffer[2] == 2);
    }
    
    #[test]
    fn swap(){
        let mut ctx = Context::new();
        run_program("1 2 swap . .", &mut ctx);
        assert!(ctx.sys_buffer[0] == 1);
        assert!(ctx.sys_buffer[1] == 2);
    }
    
    #[test]
    fn add(){
        let mut ctx = Context::new();
        run_program("3 4 + .", &mut ctx);
        assert!(ctx.sys_buffer.pop().unwrap() == 7);
    }
    
    #[test]
    fn drop(){
        let mut ctx = Context::new();
        run_program("4 3 drop .", &mut ctx);
        assert!(ctx.sys_buffer.pop().unwrap() == 4);
    }
    
    #[test]
    fn pick(){
        let mut ctx = Context::new();
        run_program("5 0 pick . .", &mut ctx);
        assert!(ctx.sys_buffer[0] == 5);
        assert!(ctx.sys_buffer[1] == 5);
        ctx.sys_buffer.clear();
        
        run_program("3 5 0 pick . . .", &mut ctx);
        assert!(ctx.sys_buffer[0] == 3);
        assert!(ctx.sys_buffer[1] == 5);
        assert!(ctx.sys_buffer[2] == 3);
    }
    
    #[test]
    #[should_panic]
    fn pick_with_no_stack(){
        let mut ctx = Context::new();
        run_program("5 2 pick", &mut ctx);
    }
}