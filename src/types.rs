use std::{any::Any, rc::Rc};

use crate::context::WorkspaceContext;

/// Forth value
#[derive(Debug, Clone)]
pub enum ForthVal{
    Null,
    // immediate
    Int(i64),
    Float(f64),
    Str(String),
    // Symbol
    Sym(String),
    // System call
    Sys(String),
    // List
    List(Rc<Vec<ForthVal>>),
    // Generator
    // A line
    Vector(Rc<Vec<ForthVal>>),
    // Compiled program
    Func(usize)
}

pub type IntOp = fn(&i64, &i64) -> i64;
pub type FloatOp = fn(&f64, &f64) -> f64;

impl ForthVal{
    /// Get formatted string
    pub fn to_string(&self) -> String{
        match self{
            ForthVal::Null => "None".to_string(),
            ForthVal::Int(v) => format!("{}", v),
            ForthVal::Float(f) => format!("{}", f),
            ForthVal::Str(s) => format!("'{}'", s),
            ForthVal::Sym(s) => format!("{}", s),
            ForthVal::Sys(s) => format!(".{}", s),
            _ => format!("Can't print")
        }
    }
    
    /// Add
    pub fn operate(&self, other: &ForthVal, fi: IntOp, ff: FloatOp) -> ForthRet{
        match self{
            ForthVal::Int(a) => {
                match other{
                    ForthVal::Int(b) => Ok(ForthVal::Int(fi(a, b))),
                    ForthVal::Float(b) => Ok(ForthVal::Int(fi(a, &(b.round() as i64)))),
                    _ => Err(ForthErr::ErrString("Can't add with int".to_string()))
                }
            },
            ForthVal::Float(a) => {
                match other{
                    ForthVal::Int(b) => Ok(ForthVal::Float(ff(a, &(*b as f64)))),
                    ForthVal::Float(b) => Ok(ForthVal::Float(ff(a, b))),
                    _ => Err(ForthErr::ErrString("Can't add with int".to_string()))
                }
            }
            _ => Err(ForthErr::ErrString("Can only do int addition".to_string()))
        }
    }
}

/// Error enumerator
#[derive(Debug)]
pub enum ForthErr{
    ErrString(String),
    ErrForthVal(ForthVal)
}

pub type ForthArgs = Vec<ForthVal>;
pub type ForthRet = Result<ForthVal, ForthErr>;

