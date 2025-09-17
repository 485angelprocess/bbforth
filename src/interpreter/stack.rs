use crate::{drivers::Serial, types::ForthVal};
use crate::drivers::DeviceInterface;

#[derive(Clone)]
struct Reg{
    base: u32,
    offset: u32,
    max: u32
}

impl Default for Reg{
    fn default() -> Self {
        Self{
            base: 32,
            offset: 0,
            max: 16
        }
    }
}

#[derive(Clone)]
pub struct Stack{
    stack: Vec<ForthVal>,
    pub local: bool,
    
    reg: Reg,
    serial: Serial
}

impl Stack{
    pub fn new(s: Serial) -> Self{
        Self{
            stack: Vec::new(),
            local: true,
            reg: Reg::default(),
            serial: s
        }
    }
    
    pub fn push(&mut self, v: ForthVal){
        if self.local{
            self.stack.push(v);
        }
        else{
            if self.reg.offset < self.reg.max{
                let data = v.to_int();
                match data{
                    Ok(d) => {
                        self.serial.write(self.reg.base+self.reg.offset, d as u32);
                        self.reg.offset += 4;
                    },
                    Err(_e) => {println!("Invalid value");}
                };
            }
            else{
                println!("Stack overflow");
            }
        }
    }
    
    pub fn pop(&mut self) -> Option<ForthVal>{
        if self.local{
            self.stack.pop()
        }
        else{
            if self.reg.offset > 0{
                self.reg.offset -= 4;
                println!("Reading");
                return match self.serial.read(self.reg.base+self.reg.offset){
                    Ok(v) => Some(ForthVal::Int(v as i64)),
                    Err(_) => None
                };
            }
            else{
                println!("-- Stack empty");
            }
            None
        }
    }
    
    pub fn len(&self) -> usize{
        if self.local{
            self.stack.len()
        }
        else{
            todo!("Client len not implemented");
        }
    }
    
    pub fn clear(&mut self){
        if self.local{
            self.stack.clear();
        }
        else{
            todo!("Clear not implemented for client stack");
        }
    }
    
    pub fn get(&self, i: usize) -> Option<&ForthVal>{
        if self.local{
            self.stack.get(i)
        }
        else{
            todo!("Get not implemented on client stack");
        }
    }
    
    pub fn last(&self) -> Option<&ForthVal>{
        if self.local{
            self.stack.last()
        }
        else{
            todo!("Last not implemented on client stack");
        }
    }
    
    pub fn get_local(&self) -> &Vec<ForthVal>{
        &self.stack
    }
}