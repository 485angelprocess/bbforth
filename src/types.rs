use std::{any::Any, fmt::Write, rc::Rc};

use regex::Error;

use crate::{context::{ForthRoutine, WorkspaceContext}, generator::GeneratorUnit, math};

/// Forth value
#[derive(Clone)]
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
    List(Vec<ForthVal>),
    // Generator
    // A line
    Vector(Rc<Vec<ForthVal>>),
    // For information about a function
    Meta(String),
    Generator(GeneratorUnit),
    Callable(Rc<ForthRoutine>),
    // Compiled program
    Func(usize)
}

impl std::fmt::Debug for ForthVal{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}

pub type IntOp = fn(&i64, &i64) -> i64;
pub type FloatOp = fn(&f64, &f64) -> f64;

fn operate_list(contents: &Vec<ForthVal>, other: &ForthVal, fi: IntOp, ff: FloatOp, reverse: bool) -> ForthRet{
    let mut result = Vec::new();
    match other{
        ForthVal::List(other_contents) => {
            // Operate on two lists, truncates to the shortest list
            let n = std::cmp::min(contents.len(), other_contents.len());
            for i in 0..n{
                if reverse{
                    result.push(other_contents[i].operate(&contents[i], fi, ff).unwrap());
                }
                else{
                    result.push(contents[i].operate(&other_contents[i], fi, ff).unwrap());
                }
            }
        },
        _ => {
            for c in contents{
                if reverse{
                    result.push(other.operate(c, fi, ff).unwrap());
                }
                else{
                    result.push(c.operate(other, fi, ff).unwrap());
                }
            }
        }
    };
    Ok(ForthVal::List(result))
}

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
            ForthVal::List(v) => format!("{:?}", v),
            ForthVal::Meta(v) => format!("Function {}", v),
            ForthVal::Generator(gen) => {
                let mut g = gen.clone();
                let mut v = String::new();
                v.push_str("{");
                for i in 0..9{
                    v.push_str(format!("{}, ", g.next().to_string()).as_str());
                }
                v.push_str(" ... }");
                v
            }
            _ => format!("Can't print")
        }
    }
    
    pub fn to_int(&self) -> Result<i64, ForthErr>{
        match self{
            ForthVal::Int(v) => Ok(v.clone()),
            _ => Err(ForthErr::ErrForthVal(self.clone()))
        }
    }
    
    pub fn to_float(&self) -> Result<f64, ForthErr>{
        match self{
            ForthVal::Float(f) => Ok(f.clone()),
            _ => Err(ForthErr::ErrForthVal(self.clone()))
        }
    }
    
    /// Add
    pub fn operate(&self, other: &ForthVal, fi: IntOp, ff: FloatOp) -> ForthRet{
        match self{
            ForthVal::Int(a) => {
                match other{
                    ForthVal::Int(b) => Ok(ForthVal::Int(fi(a, b))),
                    ForthVal::Float(b) => Ok(ForthVal::Int(fi(a, &(b.round() as i64)))),
                    ForthVal::List(b) =>  operate_list(b, other, fi, ff, true),
                    ForthVal::Generator(gen) => {
                      let mut gn = gen;
                      Ok(ForthVal::Generator(
                          gn.clone()
                            .push(self)
                            .push(&ForthVal::Callable(Rc::new(
                                ForthRoutine::Prim(
                                    math::binary_op(fi, ff)
                                )
                            ))).clone()
                        )) 
                    },
                    _ => Err(ForthErr::ErrString("Can't add with int".to_string()))
                }
            },
            ForthVal::Float(a) => {
                match other{
                    ForthVal::Int(b) => Ok(ForthVal::Float(ff(a, &(*b as f64)))),
                    ForthVal::Float(b) => Ok(ForthVal::Float(ff(a, b))),
                    ForthVal::List(b) =>  operate_list(b, other, fi, ff, true),
                    ForthVal::Generator(gen) => {
                      let mut gn = gen;
                      Ok(ForthVal::Generator(
                          gn.clone()
                            .push(self)
                            .push(&ForthVal::Callable(Rc::new(
                                ForthRoutine::Prim(
                                    math::binary_op(fi, ff)
                                )
                            ))).clone()
                        )) 
                    },
                    _ => Err(ForthErr::ErrString("Can't add with int".to_string()))
                }
            },
            ForthVal::List(contents) =>{
                operate_list(contents, other, fi, ff, false)
            },
            ForthVal::Generator(gen) => {
                let mut gn = gen.clone();
                Ok(ForthVal::Generator(
                    gn.clone()
                        .push(other)
                        .push(&ForthVal::Callable(Rc::new(
                            ForthRoutine::Prim(
                                math::binary_op(fi, ff))
                            )
                        ))
                        .clone()
                ))
            }
            _ => Err(ForthErr::ErrString(format!("Can't add {:?}, {:?}", self, other)))
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

