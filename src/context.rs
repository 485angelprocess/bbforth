use std::collections::HashMap;

use crate::math;
use crate::reader::{self, ForthReader};
use crate::types::{ForthErr, ForthVal};

// function call
pub type ForthFn = fn(&mut WorkspaceContext) -> ForthVal;
pub type ForthFnGen = Box<dyn Fn(&mut WorkspaceContext) -> ForthVal>;

enum ForthRoutine{
    Prim(ForthFnGen),
    Compiled(Vec<ForthVal>)
}

/// Duplicate top of stack
fn dup(ws: &mut WorkspaceContext) -> ForthVal{
    let t = ws.last().unwrap();
    ws.push(t.clone());
    ForthVal::Null
}

/// Start definition
fn start_define(ws: &mut WorkspaceContext) -> ForthVal{
    match ws.mode{
        Mode::NORMAL => {
            ws.mode = Mode::DECLARE;
            ForthVal::Null
        },
        _ => panic!("Already making definition")
    }
}

/// End definition
fn end_define(ws: &mut WorkspaceContext) -> ForthVal{
    match ws.mode{
        Mode::COMPILE => {
            ws.mode = Mode::NORMAL;
            ForthVal::Null
        },
        _ => panic!("Not in definition")
    }
}

/// Binary arithemtic operation
//fn binary_arithmetic(a: ForthVal, b: ForthVal) -> ForthVal{
    
//}

#[derive(PartialEq)]
enum Mode{
    NORMAL,
    DECLARE,
    COMPILE
}

pub struct WorkspaceContext{
    pub stack: Vec<ForthVal>,
    pub reply: Vec<String>,
    
    pub mode: Mode,
    
    pub define_word: Option<String>,
    pub definition: Vec<ForthVal>
}

impl WorkspaceContext{
    fn new() -> Self{
        Self{
            stack: Vec::new(),
            reply: Vec::new(),
            mode: Mode::NORMAL,
            define_word: None,
            definition: Vec::new()
        }
    }
    
    /// push value to stack
    pub fn push(&mut self, v: ForthVal){
        self.stack.push(v);
    }
    
    /// pop value from stack
    pub fn pop(&mut self) -> Option<ForthVal>{
        self.stack.pop()
    }

    /// read top of stack
    fn last(&self) -> Option<&ForthVal>{
        self.stack.last()
    }
    
    /// read n of stack
    fn peek(&self, index: usize) -> Option<&ForthVal>{
        self.stack.get(index)
    }
    
    /// length of stack
    fn len(&self) -> usize{
        self.stack.len()
    }
}

/// Forth workspace context
pub struct Workspace{
    ctx: WorkspaceContext,
    pub lookup: HashMap<String, usize>,
    pub library: HashMap<usize, ForthRoutine>
}

impl Workspace{
    pub fn new() -> Self{
        Self{
            ctx: WorkspaceContext::new(),
            lookup: HashMap::new(),
            library: HashMap::new()
        }
    }
    
    /// Insert new definition
    pub fn insert(&mut self, s: &str, f: ForthFn) -> usize{
        let id = self.lookup.len();
        self.lookup.insert(s.to_string(), id);
        self.library.insert(id, 
            ForthRoutine::Prim(
                Box::new(f)
            )
        );
        id
    }
    
    pub fn insert_box(&mut self, s: &str, f: ForthFnGen) -> usize{
        let id = self.lookup.len();
        self.lookup.insert(s.to_string(), id);
        self.library.insert(id, 
            ForthRoutine::Prim(f)
        );
        id
    }
    
    pub fn standard() -> Self{
        let mut s = Self::new();
        s.setup();
        s
    }
    
    pub fn setup(&mut self){
        self.insert(
            "dup",
            dup
        );
        self.insert(
            "clear",
            |ws| {
                ws.stack.clear();
                ForthVal::Null
            }
        );
        
        // TODO surely there is some easy way to compress these
        self.insert_box("+", math::binary_op(|a, b|{b+a}, |a, b|{b+a}));
        self.insert_box("-", math::binary_op(|a, b|{b-a}, |a, b|{b-a}));
        self.insert_box("*", math::binary_op(|a, b|{b*a}, |a, b|{b*a}));
        self.insert_box("/", math::binary_op(|a, b|{b/a}, |a, b|{b/a}));
        
        self.insert(":",start_define);
        self.insert(";", end_define);
    }
    
    /// Read line from interpreter
    pub fn read(&mut self, s: &str) -> Result<Vec<String>, ForthErr>{
        let mut reader = reader::ForthReader::from_line(s);
        self.ctx.reply.clear();
        while let Ok(v) = reader.next(){
            match self.ctx.mode{
                Mode::NORMAL => self.run(&v),
                Mode::DECLARE => {
                    // Set new word definition
                    self.ctx.definition.clear();
                    
                    match v{
                        ForthVal::Sym(s) => {
                            self.ctx.define_word = Some(s.clone());
                        }
                        _ => {
                            return Err(ForthErr::ErrString(
                                format!("Invalid definition token {}", v.to_string()
                            )))
                        }
                    }
                    
                    self.ctx.mode = Mode::COMPILE;
                }
                Mode::COMPILE => {
                    match v{
                        ForthVal::Sym(s) => {
                            if s == ";".to_string(){
                                // End definition
                                let id = self.lookup.len();
                                self.lookup.insert(self.ctx.define_word.clone().unwrap().clone(), id);
                                self.library.insert(id, ForthRoutine::Compiled(
                                    self.ctx.definition.clone()
                                ));
                                self.ctx.definition.clear();
                                self.ctx.mode = Mode::NORMAL;
                            }
                            else{
                                // TODO add arguments
                                let id = self.lookup[&s];
                                // TODO push things like ints
                                self.ctx.definition.push(ForthVal::Func(id));
                            }
                        },
                        _ => {
                            return Err(ForthErr::ErrString(
                                format!("Invalid compiled token {}", v.to_string())
                            ))
                        }
                    }
                }
            }
        }
        Ok(self.ctx.reply.clone())
    }
    
    /// Read things from a forth line
    fn run(&mut self, val: &ForthVal){
        // TODO make this reply more detailed
        // with like character positions
        match val{
            ForthVal::Int(v) => {
                self.ctx.push(ForthVal::Int(*v));
            },
            ForthVal::Float(v) =>{
                self.ctx.push(ForthVal::Float(*v))
            },
            ForthVal::Sys(s) => {
                // system call
                match s.as_str(){
                    // TODO decide/check if this clears stack
                    "" => {
                        if let Some(v) = self.ctx.pop(){
                            self.ctx.reply.push(v.to_string());
                        }
                        else{
                            self.ctx.reply.push("Stack empty".to_string());
                        }
                    },
                    "s" => {
                        while let Some(v) = self.ctx.pop(){
                            self.ctx.reply.push(v.to_string());
                        }  
                    },
                    _ => {
                        panic!("Unknown system call {}", s);
                    }
                }
            },
            ForthVal::Sym(s) => {
                if let Some(id) = self.lookup.get(s){
                    let routine = &self.library[id];
                    
                    // run function
                    // TODO put into separate function
                    match routine{
                        ForthRoutine::Prim(f) => {
                            // Primitive words can be called directly
                            let result = f(&mut self.ctx);
                            match result{
                                ForthVal::Null => (),
                                _ => self.ctx.push(result)
                            };
                        },
                        ForthRoutine::Compiled(program) => {
                            for p in program.clone(){
                                self.run(&p);
                            }
                        }
                    };
                    
                }
                else{
                    panic!("Unknown function {}", s);
                }
            },
            ForthVal::Func(f) => {
                match &self.library[f]{
                    ForthRoutine::Prim(f) => {
                        let result = f(&mut self.ctx);
                        match result{
                            ForthVal::Null => (),
                            _ => self.ctx.push(result)
                        }
                    },
                    ForthRoutine::Compiled(program) => {
                        for p in program.clone(){
                            self.run(&p);
                        }
                    }
                }
            }
            _ => todo!("Unimplemented type: {}", val.to_string())
        }
    }
}

#[cfg(test)]
mod tests{
    use super::Workspace;
    
    // Arithmetic
    #[test]
    fn add_int(){
        let mut ws = Workspace::standard();
        let result = ws.read("1 2 + .").expect("Response");
        assert_eq!(result[0], "3");
    }
    
    #[test]
    fn sub_int(){
        let mut ws = Workspace::standard();
        let result = ws.read("3 5 - .").expect("Response");
        assert_eq!(result[0], "-2");
    }
    
    #[test]
    fn mul_int(){
        let mut ws = Workspace::standard();
        let result = ws.read("3 4 * .").expect("Response");
        assert_eq!(result[0], "12");
    }
    
    #[test]
    fn div_int(){
        let mut ws = Workspace::standard();
        let result = ws.read("10 5 / .").expect("Response");
        assert_eq!(result[0], "2");
    }
    
    #[test]
    fn add_float(){
        let mut ws = Workspace::standard();
        let result = ws.read("2.2 1.5 + .").expect("Response");
        assert_eq!(result[0], "3.7");
    }
    
    #[test]
    fn define(){
        let mut ws = Workspace::standard();
        let _ = ws.read(": square dup * ;").expect("Response");
        let result = ws.read("5 square .").expect("Response");
        assert_eq!(result[0], "25");
    }
}