/*********************************/
/* Alternate mode object *********/
/* This is for defining like prefix method */
/* defininitions, assembly */
/*********************************/
use crate::types::{ForthVal, ForthErr};
use crate::interpreter::{WorkspaceContext, ForthRoutine};

#[derive(Debug)]
pub enum AltMode{
    CONTINUE,
    NEXT,
    DONE
}

struct AltTrait{
    pub comments: bool,
    pub compiled: bool,
    pub consumes_stack: usize
}

pub trait AltMethod{
    /// Read next token in buffer
    /// Returns Ok(NEXT) to clear buffer (next group of arguments)
    /// Returns Ok(CONTINUE) to keep getting values in buffer (for multiple argument)
    /// Returns Ok(DONE) to indicate method is finished reading
    fn consume(&mut self, ws: &WorkspaceContext, tokens: &Vec<ForthVal>, out: &mut Vec<ForthVal>) -> Result<AltMode, ForthErr>{
        Err(ForthErr::ErrString(format!("Consume not implemented")))
    }
    
    /// make changes to workspace
    fn finish(&self, ws: &mut WorkspaceContext, word: &String, built: &Vec<ForthVal>) -> Result<(), ForthErr>{
        Err(ForthErr::ErrString(format!("Finish unimplemented")))
    }
    
    fn traits(&self) -> AltTrait{
        AltTrait{
            comments: true,
            compiled: true,
            consumes_stack: 0
        }
    }
}

enum DefinitionMode{
    Define,
    Comment,
    Compile
}

const COMMENT_ENTRY: &str = "(";
const COMMENT_EXIT: &str = ")";

/// Object for doing any prefix operations
/// Should be able to do most cases
/// Specific operations are defined by
/// AltMethod object, which are now kind of plugins
pub struct AltCollect{
    word: Option<String>,
    mode: DefinitionMode,
    buffer: Vec<ForthVal>,
    built: Vec<ForthVal>,
    method: Box<dyn AltMethod>,
    traits: AltTrait,
    entry: bool
}

impl AltCollect{
    pub fn new(method: Box<dyn AltMethod>) -> Self{
        let traits = method.traits();
        Self{
            word: None,
            mode: DefinitionMode::Define,
            buffer: Vec::new(),
            built: Vec::new(),
            method: method,
            traits: traits,
            entry: false
        }
    }
    
    pub fn consume_stack(&mut self, ws: &mut WorkspaceContext) -> Result<AltMode, ForthErr>{
        for _i in 0..self.traits.consumes_stack{
            self.buffer.push(ws.pop().unwrap());
        }
        self.method.consume(ws, &self.buffer, &mut self.built)
    }
    
    pub fn next(&mut self, ws: &mut WorkspaceContext, token: &ForthVal) -> Result<AltMode, ForthErr>{
        if self.entry{
            // On first word, check if there is a comment entry
            self.entry = false;
            if self.traits.comments && matches(token, COMMENT_ENTRY){
                // Next things are comments
                self.mode = DefinitionMode::Comment;
                return Ok(AltMode::CONTINUE);
            }
        }
        
        match self.mode {
            DefinitionMode::Define => {
                if let ForthVal::Sym(word) = token{
                    match self.consume_stack(ws){
                        Err(e) => {return Err(e);},
                        _ => ()
                    };
                    // restricting here that the symbol following 
                    // the alt entry point is always the name of it
                    // no reason to believe its not
                    self.word = Some(word.clone()); // get the name of this definition
                    
                    println!("Defining word {:?}", self.word);
                    
                    if !self.traits.compiled{
                        // Nothing to compile (i.e. arg definition)
                        return Ok(AltMode::DONE);
                    }
                    else{
                        // Things to compile (i.e. word definition)
                        self.entry = true;
                        self.mode = DefinitionMode::Compile;
                        return Ok(AltMode::CONTINUE);
                    }
                }
            },
            DefinitionMode::Compile => {
                self.buffer.push(token.clone());
                // essentially transform buffer into new tokens
                match self.method.consume(&ws, &self.buffer, &mut self.built){
                    Ok(AltMode::NEXT) => self.buffer.clear(),
                    Ok(AltMode::DONE) => {return Ok(AltMode::DONE);},
                    Ok(AltMode::CONTINUE) => (),
                    Err(e) => {return Err(e);}
                }
                
            },
            DefinitionMode::Comment => {
                todo!("Comments not implemented")
            }
        }
        Ok(AltMode::CONTINUE)
    }
    
    pub fn finish(&mut self, ws: &mut WorkspaceContext) -> Result<(), ForthErr>{
        match &self.word{
            Some(w) => self.method.finish(ws, w, &self.built),
            None => Err(ForthErr::ErrString(format!("Undefined alt mode")))
        }
    }
}

fn matches(t: &ForthVal, s: &str) -> bool{
    if let ForthVal::Sym(v) = t{
        return v == s;
    }
    return false;
}

fn compiled_token(ws: &WorkspaceContext, t: &ForthVal) -> Result<ForthVal, ForthErr>{
    match t{
        ForthVal::Sym(s) => {
            match ws.dictionary.get_id(s){
                Some(id) => {return Ok(ForthVal::Func(*id))},
                None => {return Err(ForthErr::ErrString(format!("Word not found {}", s)));}
            }
        },
        _ => {
            Ok(t.clone())
        }
    }
}

#[derive(Default)]
pub struct DefineWord{}

impl AltMethod for DefineWord{
    fn consume(&mut self, ws: &WorkspaceContext, tokens: &Vec<ForthVal>, out: &mut Vec<ForthVal>) -> Result<AltMode, ForthErr> {
        for t in tokens{
            if matches(t, ";"){
                return Ok(AltMode::DONE);
            }
            match compiled_token(ws, t){
                Ok(ForthVal::Null) => (),
                Err(e) => {return Err(e);},
                Ok(v) => {out.push(v)}
            }
        }
        Ok(AltMode::NEXT)
    }
    
    fn finish(&self, ws: &mut WorkspaceContext, word: &String, built: &Vec<ForthVal>) -> Result<(), ForthErr> {
        if built.len() == 0{
            return Err(ForthErr::ErrString(format!("Empty definition: {}", word)));
        }
        ws.dictionary.insert_routine(word, ForthRoutine::Compiled(built.clone()));
        Ok(())
    }
}

#[derive(Default)]
pub struct Const{}

impl AltMethod for Const{
    fn consume(&mut self, _ws: &WorkspaceContext, tokens: &Vec<ForthVal>, out: &mut Vec<ForthVal>) -> Result<AltMode, ForthErr> {
        out.push(tokens[0].clone());
        Ok(AltMode::DONE)
    }
    
    fn finish(&self, ws: &mut WorkspaceContext, word: &String, built: &Vec<ForthVal>) -> Result<(), ForthErr> {
        if built.len() == 0{
            return Err(ForthErr::ErrString(format!("Empty definition: {}", word)));
        }
        ws.dictionary.insert_routine(word, ForthRoutine::Compiled(built.clone()));
        Ok(())
    }
    
    fn traits(&self) -> AltTrait {
        AltTrait{
            comments: false,
            compiled: false,
            consumes_stack: 1
        }
    }
}