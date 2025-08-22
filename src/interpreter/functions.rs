use std::{rc::Rc, thread, time::Duration};

use crate::{drivers::Serial, interpreter::WorkspaceContext, types::{ForthErr, ForthRet, ForthVal}};

use super::{math, ForthRoutine, GenEnv, Generator, GeneratorUnit, Mode, Natural, Workspace};

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

/// Start definition
pub fn start_define(ws: &mut WorkspaceContext) -> ForthVal{
    match ws.mode{
        Mode::NORMAL => {
            ws.mode = Mode::DECLARE;
            ForthVal::Null
        },
        _ => panic!("Already making definition")
    }
}

/// End definition
pub fn end_define(ws: &mut WorkspaceContext) -> ForthVal{
    ws.dictionary.insert_routine(&ws.define_word.as_ref().unwrap().clone(), 
        ForthRoutine::Compiled(
                ws.definition.clone()
    ));
    ws.definition.clear();
    ws.mode = Mode::NORMAL;
    ForthVal::Null
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

impl Workspace{
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
            "=",
            |ws|{
                let v = ws.pop().unwrap().clone();
                ws.compile(&v);
                ws.mode = Mode::ASSIGN;
                ForthVal::Null
            }
        );
        
        dict.insert(
            "{",
            |ws|{
                ws.form_builder.clear();
                ws.mode = Mode::FORM;
                ForthVal::Null
            }
        );
        
        dict.insert(
            "}",
            |ws|{
                ws.mode = Mode::NORMAL;
                ForthVal::Form(
                    ws.form_builder.clone()
                )   
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
            "remove",
            |ws|{
                let v = ws.pop().unwrap().clone();
                match v{
                    ForthVal::Meta(m) => {
                        if ws.args.contains_key(&m){
                            ws.args.remove(&m);
                        }
                        ForthVal::Null
                    },
                    _ => {
                        ForthVal::Null
                    }
                }
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
        
        dict.insert("to_int",
            |ws|{
                // TODO define unary math more generally
                to_int(&ws.pop().unwrap()).unwrap()
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
                ForthVal::List(result)
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
        
        // Basic generators
        dict.insert_generator::<Natural>("natural");
        
    }
}