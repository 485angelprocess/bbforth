use std::collections::HashMap;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

use rustyline::completion::Completer;

use crate::drivers::Serial;
use crate::generator::*;
use crate::math;
use crate::reader::{self};
use crate::types::{ForthErr, ForthVal};

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use crate::audio::AudioContext;

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

/// Duplicate top of stack
fn dup(ws: &mut WorkspaceContext) -> ForthVal{
    let t = ws.last().unwrap();
    ws.push(t.clone());
    ForthVal::Null
}

fn generator<T: Generator + Default + 'static>(ws: &mut WorkspaceContext) -> ForthVal{
    let mut gu = GeneratorUnit{
        env: GenEnv::default(),
        gen: Box::new(T::default()),
        trace: Vec::new(),
        ws: Workspace::new()
    };
    gu.consume(ws);
    ForthVal::Generator(gu)
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
    ws.dictionary.insert_routine(&ws.define_word.as_ref().unwrap().clone(), 
        ForthRoutine::Compiled(
                ws.definition.clone()
    ));
    ws.definition.clear();
    ws.mode = Mode::NORMAL;
    ForthVal::Null
}

#[derive(PartialEq, Debug, Clone)]
enum Mode{
    NORMAL,
    DECLARE,
    COMPILE,
    NEEDS,
    DEFINE,
    DOCUMENT,
    CONDITION
}

#[derive(Clone)]
pub struct Dictionary{
    pub lookup: HashMap<String, usize>,
    pub library: HashMap<usize, ForthRoutine>,
}

impl Dictionary{
    fn new() -> Self{
        Self{
            lookup: HashMap::new(),
            library: HashMap::new()
        }
    }
    
    pub fn len(&self) -> usize{
        self.lookup.len()
    }
    
    pub fn get_val(&self, s: &String) -> Option<ForthVal>{
        let f = self.library.get(self.lookup.get(s).unwrap()).unwrap();
        match f{
            ForthRoutine::Prim(_p) => Some(ForthVal::Callable(f.clone())),
            ForthRoutine::Compiled(v) => Some(ForthVal::Vector(v.clone()))
        }
    }
    
    /// Insert new definition
    pub fn insert(&mut self, s: &str, f: ForthFn) -> usize{
        self.insert_routine(&s.to_string(), ForthRoutine::Prim(Rc::new(f)))
    }
    
    /// Insert function pointer
    pub fn insert_ptr(&mut self, s: &str, f: ForthFnGen) -> usize{
        self.insert_routine(&s.to_string(), ForthRoutine::Prim(f))
    }
    
    /// Insert generator object
    fn insert_generator<T: Generator + Default + 'static>(&mut self, s: &str) -> usize{
        self.insert_routine(&s.to_string(), ForthRoutine::Prim(Rc::new(|ws| generator::<T>(ws))))
    }
    
    /// Insert routine
    fn insert_routine(&mut self, s: &String, f: ForthRoutine) -> usize{
        let id = match self.lookup.get(s){
            Some(v) => v.clone(),
            None => self.lookup.len()
        };
        
        self.lookup.insert(s.clone(), id);
        self.library.insert(id, f);
        id
    }
}

pub struct WorkspaceContext{
    pub stack: Vec<ForthVal>,
    pub reply: Vec<ForthVal>,
    
    // For declaring new words
    mode: Mode,
    pub define_word: Option<String>,
    pub definition: Vec<ForthVal>,
    
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
                       match self.dictionary.lookup.get(s){
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
    
    pub fn prompt(&self) -> &str{
        match self.ctx.mode{
            Mode::DECLARE => "dec>",
            Mode::COMPILE => ":>",
            Mode::NEEDS => "needs>",
            Mode::DEFINE =>  "def>",
            Mode::DOCUMENT => "doc>",
            Mode::CONDITION => "?>",
            _ => ">"
        }
    }
    
    /// Declare primitive functions
    pub fn setup(&mut self){
        let dict = &mut self.ctx.dictionary;
        // Stack operations
        dict.insert(
            "dup",
            dup
        );
        
        dict.insert(
            "swap",
            |ws|{
                let a = ws.pop().unwrap();
                let b = ws.pop().unwrap();
                ForthVal::Vector(vec![a, b])
            }
        );
        
        dict.insert(
            "abc_cab",
            |ws|{
                let a = ws.pop().unwrap();
                let b = ws.pop().unwrap();
                let c = ws.pop().unwrap();
                ForthVal::Vector(vec![a, c, b])
            }
        );
        
        dict.insert(
            "const",
            |ws|{
                let v = ws.pop().unwrap().clone();
                ws.compile(&v);
                ws.mode = Mode::DEFINE;
                ForthVal::Null
            }
        );
        
        dict.insert(
            "clear",
            |ws| {
                ws.stack.clear();
                ForthVal::Null
            }
        );
        
        dict.insert("if", |ws|{
            if ws.pop().unwrap().to_int().unwrap() == 0{
                ws.mode = Mode::CONDITION;
            } 
            ForthVal::Null
        });
        
        dict.insert("then", |ws|{
            if ws.mode == Mode::CONDITION{
                ws.mode = Mode::NORMAL;
            }
            ForthVal::Null
        });
        
        dict.insert(
            "needs",
            |ws|{ws.mode = Mode::NEEDS; ForthVal::Null}
        );
        
        dict.insert(
            "delay",
            |ws| {
                thread::sleep(Duration::from_millis(ws.pop().unwrap().to_int().unwrap() as u64));
                ForthVal::Null
            }
        );
        
        dict.insert(
            "==",
            |ws| {
                let a = ws.pop().unwrap();
                let b = ws.pop().unwrap();
                
                if a.to_int().unwrap() == b.to_int().unwrap(){
                    ForthVal::Int(1)
                }
                else{
                    ForthVal::Int(0)
                }
            }
        );
        
        dict.insert(
            "assert",
            |ws|{
                let msg = ws.pop().unwrap();
                let a = ws.pop().unwrap();
                if a.to_int().unwrap() > 0{
                    ForthVal::Null
                }
                else{
                    ForthVal::Err(msg.to_string())
                }
            }
        );
        
        dict.insert("play", |ws|{
            let result = ws.pop().unwrap();
           match result{
               ForthVal::Generator(gen) => {
                   ForthVal::Int(ws.audio.push(&gen) as i64)
               },
               _ => {
                   ForthVal::Err(format!("Audio channel must be generator {:?}", result))
               }
           } 
        });
        
        
        // Binary operations
        // TODO surely there is some easy way to compress these
        dict.insert_ptr("+", math::binary_op(|a, b|{b+a}, |a, b|{b+a}));
        dict.insert_ptr("-", math::binary_op(|a, b|{b-a}, |a, b|{b-a}));
        dict.insert_ptr("*", math::binary_op(|a, b|{b*a}, |a, b|{b*a}));
        dict.insert_ptr("/", math::binary_op(|a, b|{b/a}, |a, b|{b/a}));
        
        dict.insert_ptr(">", math::binary_op(|a, b|{if b>a{1} else {0}}, |a, b|{if b>a{1.0} else {0.0}}));
        dict.insert_ptr("<", math::binary_op(|a, b|{if b<a{1} else {0}}, |a, b|{if b<a{1.0} else {0.0}}));
        dict.insert_ptr("%", math::binary_op(|a, b|{b%a}, |a, b|{b%a}));
        
        dict.insert("tofloat", |ws|{
           ForthVal::Float(ws.pop().unwrap().to_float().unwrap()) 
        });
        
        dict.insert_ptr("lshift", math::binary_op(
                |a, b|{b<<a}, 
                |_a, _b|{panic!("Can't rshift floats")}));
        
        dict.insert_ptr("rshift", math::binary_op(
                |a, b|{b>>a}, 
                |_a, _b|{panic!("Can't rshift floats")}));
                
        dict.insert_ptr("&", math::binary_op(
                |a, b|{b&a}, 
                |_a, _b|{panic!("Can't rshift floats")}));
                
        dict.insert_ptr("|", math::binary_op(
                |a, b|{b|a}, 
                |_a, _b|{panic!("Can't rshift floats")}));
        
        dict.insert("access", |ws|{
           let id = ws.pop().unwrap().to_int().unwrap();
           let v = ws.pop().unwrap();
           match v{
               ForthVal::List(collection) => {
                   collection[id as usize].clone()
               },
               _ => {
                   ForthVal::Err(format!("Invalid value for collect: {:?}", v))
               }
           } 
        });
        
        // Dictionary operations
        dict.insert(":",start_define);
        // Semicolon is actualyl handled special, because it it gets close to touching
        // function pointers
        dict.insert(";", end_define);
        
        // Serial stuff
        // may make a more unified interface
        // But want to get it off the ground
        dict.insert("serial_list", 
            |_ctx|
                {Serial::print_ports()});
        
        dict.insert("serial_start",
            |ctx| {
                let port = &ctx.pop().unwrap();
                let baud = &ctx.pop().unwrap();
                ctx.serial.start(port, baud)});
                
        dict.insert("puts",
            |ctx|{
                let msg = &ctx.pop().unwrap();
                ctx.serial.put(&msg)
            });
            
        dict.insert("gets",
            |ctx|{
                ctx.serial.get()
            });
        
        dict.insert("list_to_char",
            |ctx|{
                let msg = &ctx.pop().unwrap();
                let mut result = Vec::new();
                match msg{
                    ForthVal::List(vec) => {
                        for v in vec{
                            if v.to_int().unwrap() < 127{
                                let c: char = (v.to_int().unwrap() as u8) as char;
                                result.push(ForthVal::Str(format!("{}", c)));
                            }
                        }
                        return ForthVal::List(result);
                    },
                    _ => ForthVal::Err(format!("Can't convert to char {:?}", msg))
                }
            });
        
        // Basic generators
        dict.insert_generator::<Natural>("natural");
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
                      let ending_id = self.ctx.dictionary.lookup["then"];
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
                if let Some(id) = self.ctx.dictionary.lookup.get(s){
                    let routine = &self.ctx.dictionary.library[id];
                    
                    // run function
                    return self.run_routine(&routine.clone());
                }
                else{
                    return Err(ForthErr::ErrString(format!("Unknown word {}", s)));
                }
            },
            ForthVal::Func(f) => {
                // This is for compiled functions
                let routine = self.ctx.dictionary.library.get(f).unwrap();
                return self.run_routine(&routine.clone());
            },
            ForthVal::Meta(m) =>{
                // Use backtick character to get information about functions
                if let Some(id) = self.ctx.dictionary.lookup.get(m){
                    match &self.ctx.dictionary.library[id]{
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
                else{
                    self.ctx.reply.push(ForthVal::Str(format!("Unknown element {}", m)));
                }
                self.ctx.push(val.clone());
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