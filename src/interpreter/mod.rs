use std::collections::HashMap;
use std::rc::Rc;

use crate::drivers::Serial;
use crate::generator::*;
use crate::reader::{self};
use crate::types::{ForthErr, ForthVal};

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use crate::audio::AudioContext;

mod functions;
mod dictionary;
pub mod math;

use dictionary::*;
use functions::*;

// function call
pub type ForthFn = fn(&mut WorkspaceContext) -> ForthVal;
pub type ForthFnGen = Rc<dyn Fn(&mut WorkspaceContext) -> ForthVal>;

// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[derive(Clone)]
pub enum ForthRoutine{
    Prim(ForthFnGen),
    Compiled(Vec<ForthVal>)
}

#[derive(PartialEq, Debug, Clone)]
enum Mode{
    NORMAL,
    DECLARE,
    COMPILE,
    NEEDS,
    DEFINE,
    DOCUMENT,
    CONDITION,
    ASSIGN,
    FORM,
    FORM_DEFINE
}

enum Namespace{
    MAIN,
    CLIENT
}

pub struct WorkspaceContext{
    pub stack: Vec<ForthVal>,
    pub reply: Vec<ForthVal>,
    
    // For declaring new words
    mode: Mode,
    pub define_word: Option<String>,
    pub definition: Vec<ForthVal>,
    pub args: HashMap<String, ForthVal>,
    pub form_builder: HashMap<String, ForthVal>,
    pub form_word: String,
    pub dictionary: Dictionary,
    
    pub audio: AudioContext,
    
    pub serial: Serial,
}

impl WorkspaceContext{
    fn new() -> Self{
        Self{
            stack: Vec::new(),
            reply: Vec::new(),
            mode: Mode::NORMAL,
            define_word: None,
            args: HashMap::new(),
            
            form_word: String::new(),
            form_builder: HashMap::new(),
            
            definition: Vec::new(),
            dictionary: Dictionary::new(),
            audio: AudioContext::new(),
            serial: Serial::new()
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
    
    /// Add to compile vector
    pub fn compile(&mut self, v: &ForthVal){
        match v{
            ForthVal::Sym(s) => {
                // Get function pointers instead of pushing strings
                // to compiled functions
                match s.as_str(){
                    ";" => {let _ = end_define(self);},
                    "(" => {self.mode = Mode::DOCUMENT;},
                    ")" => {self.mode = Mode::COMPILE;},
                    _ => {
                       match self.dictionary.get_id(s){
                        Some(id) => self.definition.push(ForthVal::Func(*id)),
                        None => println!("Word not found {}", s)
                       };
                    }
                };
            },
            ForthVal::Null => {},
            _ => {
                self.definition.push(v.clone())
            }
        }
    }
    
    pub fn define_form_field(&mut self, value: &ForthVal){
        let name = self.form_word.clone();
        
        match value{
            ForthVal::Null => todo!("Handle invalid field name"),
            _ => {
                self.form_builder.insert(name, value.clone());
            }
        };
    }
}

/// Forth workspace context
pub struct Workspace{
    pub ctx: WorkspaceContext
}

impl Workspace{
    pub fn new() -> Self{
        Self{
            ctx: WorkspaceContext::new()
        }
    }
    
    pub fn standard() -> Self{
        let mut s = Self::new();
        s.setup();
        s
    }
    
    /// Display prompt based on context
    pub fn prompt(&self) -> &str{
        match self.ctx.mode{
            Mode::DECLARE => "dec>",
            Mode::COMPILE => ":>",
            Mode::NEEDS => "needs>",
            Mode::DEFINE =>  "def>",
            Mode::DOCUMENT => "doc>",
            Mode::CONDITION => "?>",
            _ => {
                if self.ctx.dictionary.is_local(){
                    "riscv>"
                }
                else{
                    ">"
                }
            }
        }
    }
    
    fn interpret_token(&mut self, v: &ForthVal) -> Result<(), ForthErr>{
        match self.ctx.mode{
            Mode::NORMAL => self.run(v),
            Mode::CONDITION => {
                match v{
                    ForthVal::Sym(s) =>{
                        if s == "then"{
                            self.ctx.mode = Mode::NORMAL;
                        }
                        Ok(())
                    },
                    ForthVal::Func(id) =>{
                      let ending_id = self.ctx.dictionary.then_id();
                      if *id == ending_id{
                          self.ctx.mode = Mode::NORMAL;
                      }
                      Ok(())
                    },
                    _ => {
                        // Ignore
                        Ok(())
                    }
                }  
            },
            Mode::DEFINE => {
                match v{
                    ForthVal::Sym(s) =>{
                        self.ctx.define_word = Some(s.clone());
                        end_define(&mut self.ctx);
                        Ok(())
                    },
                    _ => {
                        return Err(ForthErr::ErrString(
                            format!("Invalid definition token {}", v.to_string()
                        )))
                    }
                }  
            },
            Mode::ASSIGN => {
              match v{
                  ForthVal::Sym(s) =>{
                      self.ctx.args.insert(s.clone(), self.ctx.definition[0].clone());
                      self.ctx.definition.clear();
                      self.ctx.mode = Mode::NORMAL;
                      Ok(())
                  },
                  _ => {
                      return Err(ForthErr::ErrString(
                          format!("Invalid var name {}", v.to_string()
                    )))
                  }
              }  
            },
            Mode::FORM => {
                match v{
                    ForthVal::Sym(s) => {
                        if s == "}"{
                            self.ctx.push(ForthVal::Form(
                                self.ctx.form_builder.clone()
                            ));
                            self.ctx.form_builder.clear();
                            self.ctx.mode = Mode::NORMAL;
                            Ok(())
                        }
                        else if s.starts_with(":"){
                            let ms = s.clone();
                            self.ctx.form_word = ms.strip_prefix(":").unwrap().to_string();
                            self.ctx.mode = Mode::FORM_DEFINE;
                            Ok(())
                        }
                        else{
                            return Err(ForthErr::ErrString(
                                format!("Invalid field {:?}", v)
                            ));
                        }
                    },
                    _ => {
                        return Err(ForthErr::ErrString(
                         format!("Forms must contain fields, {:?}", v)   
                        ))
                    }
                }
            },
            Mode::FORM_DEFINE => {
                let _ = self.run(v);
                self.ctx.mode = Mode::FORM;
                let v = self.ctx.pop().unwrap();
                self.ctx.define_form_field(&v);
                Ok(())
            },
            Mode::DECLARE => {
                // Set new word definition
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
                Ok(())
            },
            Mode::COMPILE => {
                self.ctx.compile(v);
                Ok(())
            },
            Mode::DOCUMENT => {
                // temporary ignore docuemntation
                match v{
                    ForthVal::Sym(s) =>{
                        if s.as_str() == ")"{
                            self.ctx.mode = Mode::COMPILE;
                        }
                    },
                    _ => {
                        return Err(ForthErr::ErrString(format!("Invalid document {}", v.to_string())));
                    }
                }
                Ok(())  
            },
            Mode::NEEDS => {
                println!("Loading file {}", v.to_string());
                self.ctx.mode = Mode::NORMAL;
                match v{
                    ForthVal::Sym(s) => {
                        self.read_file(format!("{}.fs", v.to_string()).as_str());
                    },
                    ForthVal::Str(s) => {
                        self.read_file(format!("{}", v.to_string()).as_str());
                    },
                    _ => {return Err(ForthErr::ErrString(format!("Invalid file {:?}", v)));}
                }
                Ok(())
            }
        }
    }
    
    /// Read line from interpreter
    pub fn read(&mut self, s: &str) -> Result<Vec<ForthVal>, ForthErr>{
        let mut reader = reader::ForthReader::from_line(s);
        self.ctx.reply.clear();
        while !reader.is_done(){
            let token = reader.next();
            match token{
                Ok(v) => {
                    self.interpret_token(&v)?;
                },
                Err(err) => {
                    println!("Error {:#?}", err);
                    return Err(err)
                }
            }
        }
        Ok(self.ctx.reply.clone())
    }
    
    
    
    fn run_routine(&mut self, routine: &ForthRoutine) -> Result<(), ForthErr>{
        match routine{
            ForthRoutine::Prim(f) => {
                // Primitive words can be called directly
                let result = f.clone()(&mut self.ctx);
                match result{
                    ForthVal::Null => (),
                    ForthVal::Vector(values) =>{
                      for v in values{
                          self.ctx.push(v);
                      }  
                    },
                    ForthVal::Err(s) => {
                        return Err(ForthErr::ErrString(s.clone()))
                    }
                    _ => self.ctx.push(result)
                };
            },
            ForthRoutine::Compiled(program) => {
                for p in program.clone(){
                    self.interpret_token(&p)?;
                }
            }
        };
        Ok(())
    }
    
    /// Read things from a forth line
    pub fn run(&mut self, val: &ForthVal) -> Result<(), ForthErr>{
        // TODO make this reply more detailed
        // with like character positions
        match val{
            ForthVal::Sys(s) => {
                // system call
                // May move these all to 
                match s.as_str(){
                    // TODO decide/check if this clears stack
                    "" => {
                        if let Some(v) = self.ctx.pop(){
                            self.ctx.reply.push(v);
                        }
                        else{
                            self.ctx.reply.push(ForthVal::Str("Stack empty".to_string()));
                        }
                    },
                    "s" => {
                        for i in 0..self.ctx.len(){
                            self.ctx.reply.push(self.ctx.peek(i).unwrap().clone());
                        }
                    },
                    "x" => {
                        if let Some(v) = self.ctx.pop(){
                            self.ctx.reply.push(ForthVal::Str(format!("{:#02x}", 
                                v.to_int().unwrap())));
                        }
                        else{
                            self.ctx.reply.push(ForthVal::Str("Stack empty".to_string()));
                        }
                    },
                    "b" => {
                        if let Some(v) = self.ctx.pop(){
                            self.ctx.reply.push(ForthVal::Str(format!("{:#02b}", 
                                v.to_int().unwrap())));
                        }
                        else{
                            self.ctx.reply.push(ForthVal::Str("Stack empty".to_string()));
                        }
                    }
                    _ => {
                        return Err(ForthErr::ErrString(format!("Unknown special function {}", s)));
                    }
                }
            },
            ForthVal::Sym(s) => {
                // General symbol type
                // This is normally a  word
                if self.ctx.args.contains_key(s){
                    self.ctx.push(self.ctx.args[s].clone());
                    return Ok(());
                }
                else if let Some(routine) = self.ctx.dictionary.get_fn(s){
                    // run function
                    return self.run_routine(&routine.clone());
                }
                else{
                    return Err(ForthErr::ErrString(format!("Unknown word {}", s)));
                }
            },
            ForthVal::Func(f) => {
                // This is for compiled functions
                let routine = self.ctx.dictionary.get_fn_from_id(f).unwrap();
                return self.run_routine(&routine.clone());
            },
            ForthVal::Meta(m) =>{
                // Use backtick character to get information about functions
                if let Some(id) = self.ctx.dictionary.get_id(m){
                    match self.ctx.dictionary.get_fn_from_id(id).unwrap(){
                        ForthRoutine::Prim(_f) => {
                            self.ctx.reply.push(ForthVal::Str(format!("Builtin function {}", m)));
                            self.ctx.reply.push(ForthVal::Str(format!("Id: {}", id)));
                        },
                        ForthRoutine::Compiled(_p) => {
                            self.ctx.reply.push(ForthVal::Str(format!("User function {}", m)));
                            self.ctx.reply.push(ForthVal::Str(format!("Id: {}", id)));
                        }
                    }
                }
                self.ctx.push(val.clone());
            },
            ForthVal::Property((form, field)) => {
              match &self.ctx.args[form] {
                  ForthVal::Form(f) => {
                      self.ctx.push(f[field].clone());
                  },
                  _ => {
                      return Err(ForthErr::ErrString(format!("Invalid field {}", field)));
                  }
              };
            },
            ForthVal::Callable(m) => {
                // Function pointer
                return self.run_routine(&m.clone());
            }
            _ => self.ctx.push(val.clone())
        };
        
        Ok(())
    }
    
    /// Read forth program from file
    pub fn read_file(&mut self, filename: &str){
        if let Ok(lines) = read_lines(filename){
            for line in lines.map_while(Result::ok){
                // Do operations on input
                match self.read(line.as_str()){
                    Ok(reply) => {
                        if reply.len() > 0{
                            for r in reply{
                                print!("{} ", r.to_string());
                            }
                            print!("\n");
                        }
                    },
                    Err(err) => {
                        match err{
                            ForthErr::ErrString(s) => {
                                println!("Error: {:?}", s);
                            },
                            ForthErr::ErrForthVal(v) => {
                                println!("Error on value: {:?}", v);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests{
    use crate::types::ForthVal;

    use super::Workspace;
    
    // Arithmetic
    #[test]
    fn add_int(){
        let mut ws = Workspace::standard();
        let result = ws.read("1 2 + .").expect("Response");
        assert_eq!(result[0].to_int().unwrap(), 3);
    }
    
    #[test]
    fn sub_int(){
        let mut ws = Workspace::standard();
        let result = ws.read("3 5 - .").expect("Response");
        assert_eq!(result[0].to_int().unwrap(), -2);
    }
    
    #[test]
    fn mul_int(){
        let mut ws = Workspace::standard();
        let result = ws.read("3 4 * .").expect("Response");
        assert_eq!(result[0].to_int().unwrap(), 12);
    }
    
    #[test]
    fn div_int(){
        let mut ws = Workspace::standard();
        let result = ws.read("10 5 / .").expect("Response");
        assert_eq!(result[0].to_int().unwrap(), 2);
    }
    
    #[test]
    fn add_float(){
        let mut ws = Workspace::standard();
        let result = ws.read("2.2 1.5 + .").expect("Response");
        assert_eq!(result[0].to_float().unwrap(), 3.7);
    }
    
    #[test]
    fn define(){
        let mut ws = Workspace::standard();
        let _ = ws.read(": square dup * ;").expect("Response");
        let result = ws.read("5 square .").expect("Response");
        assert_eq!(result[0].to_int().unwrap(), 25);
    }
    
    #[test]
    fn list_add(){
        let mut ws = Workspace::standard();
        let result = ws.read("[1 2] 3 + .").expect("Response");
        match &result[0]{
            ForthVal::List(l) => {
                assert_eq!(l[0].to_int().unwrap(), 4);
                assert_eq!(l[1].to_int().unwrap(), 5);
                assert_eq!(l.len(), 2);
            },
            _ => panic!("Unexpected return {}", result[0].to_string())
        }
    }
}