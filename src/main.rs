extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

mod reader;
mod types;
mod context;
mod math;
mod generator;

use types::ForthErr;

fn main(){
    println!("__welcome__");
    
    let mut rl = Editor::<(), rustyline::history::DefaultHistory>::new().unwrap();
    
    if rl.load_history(".bee-history").is_err(){
        eprintln!("No history");
    }
    
    // Set up environment
    let mut ctx = context::Workspace::new();
    
    ctx.setup();
    
    // Main loop
    loop{
        let readline =rl.readline("> ");
        match readline{
            Ok(line) => {
                if !line.is_empty(){
                    // Add line to history
                    let _ = rl.add_history_entry(&line);
                    rl.save_history(".bee-history").unwrap();
                    
                    // Do operations on input
                    match ctx.read(line.as_str()){
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
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
