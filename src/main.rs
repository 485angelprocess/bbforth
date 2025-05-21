mod forth;

use forth::{run_program, Context};

fn main() {
    let mut ctx = Context::new();
    
    let program = "5 dup * .";
    
    run_program(program, &mut ctx);
}
