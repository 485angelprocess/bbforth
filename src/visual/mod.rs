use crate::interpreter::mem::Location;
use crate::types::ForthVal;

enum MemData{
    None,
    Typed(ForthVal),
    Single(u32),
    List(Vec<u32>)
}

struct Entry{
    loc: Location,
    value: MemData
}

pub fn decode(v: u32) -> Option<String>{
    return match rvdc::Inst::decode(v){
        Ok((inst, _compressed)) => Some(format!("{inst}")),
        _ => None
    };
}