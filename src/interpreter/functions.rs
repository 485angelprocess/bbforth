use std::{rc::Rc, thread, time::Duration};

use crate::{drivers::Serial, interpreter::WorkspaceContext, types::{ForthErr, ForthRet, ForthVal, AsmPromise}};
use crate::interpreter::alt::*;
use crate::drivers::DeviceInterface;
use crate::interpreter::mem::Location;

use crate::visual::decode;

use super::{math, Dictionary, ForthRoutine, GenEnv, Generator, GeneratorUnit, Mode, Natural, Workspace};

/// Duplicate top of stack
pub fn dup(ws: &mut WorkspaceContext) -> ForthVal{
    let t = ws.last().unwrap();
    ws.push(t.clone());
    ForthVal::Null
}

pub fn generator<T: Generator + Default + 'static>(ws: &mut WorkspaceContext) -> ForthVal{
    let mut gu = GeneratorUnit{
        env: GenEnv::default(),
        gen: Box::new(T::default()),
        trace: Vec::new(),
        ws: Workspace::new()
    };
    gu.consume(ws);
    ForthVal::Generator(gu)
}

pub fn to_int(v: &ForthVal) -> ForthRet{
    match v{
        ForthVal::Float(f) => Ok(ForthVal::Int(f.round() as i64)),
        ForthVal::Int(v) => Ok(ForthVal::Int(*v)),
        ForthVal::List(values) => {
            let mut result = Vec::new();
            for va in values{
                result.push(to_int(va)?)
            }
            Ok(ForthVal::List(result))
        },
        ForthVal::Generator(g) => {
            // girls when they write sensible code voice
            let mut gp = g.clone();
    
          gp.push(&ForthVal::Callable(ForthRoutine::Prim(Rc::new(|ws|{
              to_int(&ws.pop().unwrap()).unwrap()
          }))));
          Ok(ForthVal::Generator(gp))
        },
        _ => Err(ForthErr::ErrString(
            format!("Invalid type to int {:?}", v))
        )
    }
}

fn unwrap(ws: &mut WorkspaceContext, v: &ForthVal) -> Vec<ForthVal>{
    match v{
        ForthVal::List(values) => {
            let mut result = Vec::new();
            for lv in values{
                result.append(&mut unwrap(ws, lv));
            }
            result
        },
        _ => {
            return vec![v.clone()];
        }
    }
}

/// Define print functions
fn setup_print(dict: &mut Dictionary){
    dict.insert(".", |ws|{
        if let Some(v) = ws.pop(){
            ws.reply.push(v);
        }
        else{
            ws.reply.push(ForthVal::Str("Stack empty".to_string()));
        }
        ForthVal::Null
    });
    
    dict.insert(".s", |ws|{
        for i in 0..ws.len(){
            ws.reply.push(ws.peek(i).unwrap().clone());
        }
        ForthVal::Null
    });
    
    dict.insert(".x", |ws|{
        if let Some(v) = ws.pop(){
            ws.reply.push(ForthVal::Str(format!("{:#02x}", 
                v.to_int().unwrap())));
        }
        else{
            ws.reply.push(ForthVal::Str("Stack empty".to_string()));
        }
        ForthVal::Null
    });
    
    dict.insert(".b", |ws|{
        if let Some(v) = ws.pop(){
            ws.reply.push(ForthVal::Str(format!("{:#02b}", 
                v.to_int().unwrap())));
        }
        else{
            ws.reply.push(ForthVal::Str("Stack empty".to_string()));
        }
        ForthVal::Null
    });
}

fn setup_alt(dict: &mut Dictionary){
    dict.insert_alt_mode::<DefineWord>(":");
    dict.insert_alt_mode::<Const>("const");
    dict.insert_alt_mode::<ProcBuilder>("{");
    dict.insert_alt_mode::<Var>("=");
    dict.insert_alt_mode::<ClientVar>("#=");
}

impl Workspace{
    /// Declare primitive functions
    pub fn setup(&mut self){
        let dict = &mut self.ctx.dictionary;
        
        setup_print(dict);
        setup_alt(dict);
    
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
            "library_set",
            |ws|{
                let mode = ws.pop().unwrap();
                ws.dictionary.set_context(mode.to_int().unwrap() != 0);
                ForthVal::Null
            }
        );
        
        dict.insert(
            "stack_set",
            |ws|{
                let mode = ws.pop().unwrap();
                let m = mode.to_int().unwrap() == 0;
                ws.stack.local = m;
                ForthVal::Null
            }
        );
        
        // Stall waits for a register/memory address to be 0
        dict.insert(
            "stall",
            |ws|{
                let checking = true;
                let mut counter = 0;
                let timeout = 10;
                let addr = ws.pop().unwrap().to_int().unwrap();
                while checking{
                    let result = ws.serial.read(addr as u32);
                    if let Ok(r) = result{
                        if r == 0{
                            return ForthVal::Null;
                        } 
                    }
                    counter += 1;
                    if counter >= timeout{
                        return ForthVal::Err(format!("Timed out while stalling for state change"));
                    }
                }
                ForthVal::Null
            }
        );
        
        dict.insert(
            "stack_size",
            |ws|{
                ForthVal::Int(ws.stack.len() as i64)
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
        
        dict.insert_ptr("&", math::binary_op(|a, b|{b&a}, |_a, _b|{panic!("cant do bitwise on float")}));
        
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
        
        dict.insert("to_int",
            |ws|{
                // TODO define unary math more generally
                to_int(&ws.pop().unwrap()).unwrap()
            }
        );
        
        dict.insert("write_bin",
            |ws|{
                if let ForthVal::Str(filename) = ws.pop().unwrap(){
                    let _result = ws.mem.to_bin(&filename);
                    return ForthVal::Null;
                }
                else{
                    return ForthVal::Err(format!("Invalid data type"));
                }
            }  
        );
        
        dict.insert("collect",
            |ws|{
               let gen = ws.pop().unwrap();
               let len = ws.pop().unwrap().to_int().unwrap() as usize;
               match gen{
                   ForthVal::Generator(mut gen) => {
                       let mut result = Vec::new();
                       for _i in 0..len{
                           result.push(gen.next())
                       }
                       result.reverse();
                       ForthVal::List(result)
                   },
                    _ => ForthVal::Err(format!("Can only collect generator, got {:?}", gen))
               }
            });
            
        dict.insert("len",
            |ws|{
                let value = ws.pop().unwrap();
                match value{
                    ForthVal::List(v) => ForthVal::Int(v.len() as i64),
                    _ => ForthVal::Err(format!("Can only get len of list, got {:?}", value))
                }
            }    
        );
            
        dict.insert("repeat",
            |ws|{
                let value = ws.pop().unwrap();
                let len = ws.pop().unwrap().to_int().unwrap() as usize;
                let mut result = Vec::new();
                for _i in 0..len{
                    result.push(value.clone());
                }
                ForthVal::List(result)
            }  
        );
        
        dict.insert("stack_to_list",
            |ws|{
                let mut result = Vec::new();
                if ws.len() == 0{
                    return ForthVal::Err("Stack empty".to_string());
                }
                for _i in 0..ws.len(){
                    result.push(ws.pop().unwrap());
                }
                result.reverse();
                ForthVal::List(result)
            }
        );
        
        dict.insert("remove_from_list",
            |ws|{
                match ws.pop().unwrap(){
                    ForthVal::List(mlist) => {
                        ForthVal::List(mlist[1..mlist.len()].to_vec())
                    },
                    _ => ForthVal::Err(format!("Invalid type"))
                }
            }  
        );
        
        dict.insert("list_group",
            |ws|{
                let mut result = Vec::new();
                if let ForthVal::List(a) = ws.pop().unwrap(){
                    if let ForthVal::List(b) = ws.pop().unwrap(){
                        for i in 0..a.len(){
                            result.push(a[i].append(b[i].clone()));
                        }
                        return ForthVal::List(result);
                    }
                }
                ForthVal::Err(format!("Unsupported types to group"))
            }
        );
        
        dict.insert("unwrap",
            |ws|{
                let mlist = ws.pop().unwrap();
                ForthVal::Vector(unwrap(ws, &mlist))
            }  
        );
        
        dict.insert("read",
            |ws|{
                if let Some(addr) = ws.pop(){
                    let addr = addr.to_int().unwrap();
                    let result = ws.device.borrow_mut().read(addr as u32);
                    if let Ok(data) = result{
                        ForthVal::Int(data as i64)
                    }
                    else{
                        ForthVal::Err("Could not read addr".to_string())
                    }
                }
                else{
                    ForthVal::Err("Stack empty".to_string())
                }
            }
        );
        
        dict.insert("write",
            |ws|{
                let mut addr = ws.pop().unwrap().to_int().unwrap() as u32;
                let data = ws.pop().unwrap();
                
                match data{
                    ForthVal::Int(v) => {
                        let _result = ws.device.borrow_mut()
                        .write(addr as u32, v as u32);
                    },
                    ForthVal::List(vals) => {
                        for i in 0..vals.len(){
                            let v = vals[i].to_int().unwrap();
                            let _result = ws.device.borrow_mut().write(addr, v as u32);
                            addr += 4;
                        }
                    },
                    _ => {
                        return ForthVal::Err(format!("Unsupported data for write {:?}", data));
                    }
                };
                ForthVal::Null
            }
        );
        
        dict.insert("@", |ws|{
            let addr = match ws.pop(){
                Some(ForthVal::Var(loc)) => {
                    match loc{
                        Location::Local(a) => a,
                        Location::Client(addr, _) => {
                            return match ws.mem.access_client(addr as usize){
                                Ok(v) => v,
                                Err(v) => v
                            };
                        }
                    }
                },
                Some(ForthVal::Int(a)) => a as usize,
                _ => {
                    return ForthVal::Err("Can't use as address".to_string())
                }
            };
            if let Some(v) = ws.mem.access_local(addr){
                v.clone()
            }
            else{
                ForthVal::Err(format!("No variable at location {}", addr))
            }
        });
        
        dict.insert(
            "!",
            |ws|{
                let addr = match ws.pop(){
                Some(ForthVal::Var(loc)) => {
                    match loc{
                        Location::Local(a) => a,
                        Location::Client(addr, _) => {
                            addr as usize
                        }
                    }
                },
                Some(ForthVal::Int(a)) => a as usize,
                _ => {
                    return ForthVal::Err("Can't use as address".to_string())
                }
            };
            return ForthVal::Int(addr as i64);
            }
        );
        
        dict.insert(
            "jal%",
            |ws|{
                if let Some(ForthVal::Int(rd)) = ws.pop(){
                    if let Some(ForthVal::Meta(name)) = ws.pop(){
                        return ForthVal::Promise((name, AsmPromise::JAL(rd as u32)));
                    }
                }
                return ForthVal::Err(format!("Invalid arguments"));
            }
        );
        
        dict.insert(
            "decode",
            |ws|{
                if let Some(v) = ws.pop(){
                    match v{
                        ForthVal::Int(v) => {
                            if let Some(d) = decode(v as u32){
                                return ForthVal::Str(d);
                            }
                            else{
                                return ForthVal::Err(format!("Not valid riscv instruction"))
                            }
                        },
                        _ => {
                            return ForthVal::Err(format!("Not valid type for decode {:?}", v))
                        }
                    }
                }
                else{
                    return ForthVal::Err(format!("Stack empty"))
                }
            }
        );
        
        // Basic generators
        dict.insert_generator::<Natural>("natural");
        
    }
}