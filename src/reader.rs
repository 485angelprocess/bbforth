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
fn read_atom(token: &String) -> ForthRet{
    lazy_static!{
        static ref INT_RE: Regex = Regex::new(r"^-?[0-9]+$").unwrap();
        static ref HEX_RE: Regex = Regex::new(r"x[0-9|A-F|a-f]+$").unwrap();
        static ref BIN_RE: Regex = Regex::new(r"b[0-1]+$").unwrap();
        static ref FLOAT_RE: Regex = Regex::new(r"^-?[0-9]+.[0-9]+$").unwrap();
        static ref STR_RE: Regex = Regex::new(r#""(?:\\.|[^\\"])*""#).unwrap();
    }
    
    if INT_RE.is_match(&token){
        Ok(ForthVal::Int(token.parse().unwrap()))
    }
    else if HEX_RE.is_match(&token){
        // Convert from hex
        let wo_prefix= token.trim_start_matches("x");
        Ok(ForthVal::Int(i64::from_str_radix(wo_prefix, 16).unwrap()))
    }
    else if BIN_RE.is_match(&token){
        let wo_prefix = token.trim_start_matches("b");
        Ok(ForthVal::Int(i64::from_str_radix(wo_prefix, 2).unwrap()))
    }
    else if FLOAT_RE.is_match(&token){
        Ok(ForthVal::Float(token.parse().unwrap()))
    }
    else if STR_RE.is_match(&token){
        // TODO better unescape
        // I think forth also usually has it has a ." String " sequence
        Ok(ForthVal::Str(token[1..token.len()-1].to_string()))
    }
    else{
        let t = token.to_string();
        if t.starts_with("."){
            Ok(ForthVal::Sym(t))
        }
        else if t.contains("."){
            let sp: Vec<&str> = t.split(".").collect();
            Ok(ForthVal::Property((sp[0].to_string(), sp[1].to_string())))
        }
        else if t.starts_with("`"){
            Ok(ForthVal::Meta(t[1..].to_string()))
        }
        else{
            Ok(ForthVal::Sym(token.to_string()))
        }       
    }
}

/// List of values
fn read_list(reader: &mut ForthReader) -> ForthRet{
    let mut mlist = Vec::new();
    
    loop{
        let token = reader.get_token()?;
        
        let val = match &token[..]{
            "]" => return Ok(ForthVal::List(mlist)),
            _ => read_atom(&token)?
        };
        
        mlist.push(val);
        
        reader.step();
    }
}

/// Read meta
fn read_meta(reader: &mut ForthReader) -> ForthRet{
    Ok(ForthVal::Meta(reader.get_token()?))
}

/// Read token, called recursively
fn read_token(reader: &mut ForthReader) -> ForthRet{
    let token = reader.get_token()?;
    
    match &token[..]{
        "[" => {
            reader.step();
            read_list(reader)
        },
        "`" => {
            reader.step();
            read_meta(reader)  
        },
        "]" => Err(ForthErr::ErrString("Got end of list before start of list".to_string())),
        "\\" => {reader.comment = true; Ok(ForthVal::Null)},
        _ => read_atom(&token)
    }
}

#[derive(Debug, Clone)]
pub struct ForthReader{
    tokens: Vec<String>,
    pos: usize,
    comment: bool
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
    
    pub fn is_done(&self) -> bool{
        self.pos == self.tokens.len() || self.comment || self.tokens[0] == "\\"
    }
    
    /// Increment position
    fn step(&mut self){
        self.pos += 1;
    }
    
    pub fn from_line(s: &str) -> Self{
        Self{
            tokens: tokenize(s),
            pos: 0,
            comment: false
        }
    }
    
    pub fn is_empty(&self) -> bool{
        self.tokens.len() == 0 || self.tokens[0].starts_with('\\')
    }
    
    pub fn next(&mut self) -> ForthRet{
        let ret = read_token(self);
        self.step();
        ret
    }
}