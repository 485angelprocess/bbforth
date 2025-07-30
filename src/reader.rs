/*
Read in lines and compile/run
*/
use crate::types::{ForthErr, ForthRet, ForthVal};

use regex::Regex;
use lazy_static::lazy_static;

/// Split forth lines into tokens
fn tokenize(str: &str) -> Vec<String>{
    // TODO: make static
    lazy_static!{
        static ref TokenRegex: Regex = Regex::new(
                r###"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]+)"###
        ).unwrap();
    }
    
    let mut res = vec![];
    
    for cap in TokenRegex.captures_iter(str){
        res.push(String::from(&cap[1]))
    }
    res
}

/// Read word in
fn read_atom(reader: &mut ForthReader) -> ForthRet{
    lazy_static!{
        static ref INT_RE: Regex = Regex::new(r"^-?[0-9]+$").unwrap();
        static ref FLOAT_RE: Regex = Regex::new(r"^-?[0-9]+.[0-9]+$").unwrap();
        static ref STR_RE: Regex = Regex::new(r#""(?:\\.|[^\\"])*""#).unwrap();
    }
    
    let token = reader.get_token()?;
    
    if INT_RE.is_match(&token){
        Ok(ForthVal::Int(token.parse().unwrap()))
    }
    else if FLOAT_RE.is_match(&token){
        Ok(ForthVal::Float(token.parse().unwrap()))
    }
    else if STR_RE.is_match(&token){
        // TODO unescape
        // I think forth also usually has it has a ." String " sequence
        Ok(ForthVal::Str(token.to_string()))
    }
    else{
        let t = token.to_string();
        if t.starts_with("."){
            Ok(ForthVal::Sys(t[1..].to_string()))
        }
        else{
            Ok(ForthVal::Sym(token.to_string()))
        }       
    }
}

/// Read token, called recursively
fn read_token(reader: &mut ForthReader) -> ForthRet{
    let token = reader.get_token()?;
    
    match &token[..]{
        _ => read_atom(reader)
    }
}

#[derive(Debug, Clone)]
pub struct ForthReader{
    tokens: Vec<String>,
    pos: usize
}

impl ForthReader{
    /// Get next token
    fn get_token(&mut self) -> Result<String, ForthErr>{
        Ok(
            self.tokens
                .get(self.pos)
                .ok_or_else(|| ForthErr::ErrString("Underflow".to_string()))?
                .to_string()
        )
    }
    
    pub fn len(&self) -> usize{
        self.tokens.len()
    }
    
    /// Increment position
    fn step(&mut self){
        self.pos += 1;
    }
    
    pub fn from_line(s: &str) -> Self{
        Self{
            tokens: tokenize(s),
            pos: 0
        }
    }
    pub fn is_empty(&self) -> bool{
        self.tokens.len() == 0
    }
    
    pub fn next(&mut self) -> ForthRet{
        let ret = read_atom(self);
        self.step();
        ret
    }    
}