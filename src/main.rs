mod forth;

use forth::{run_program, Context, PRIM};

fn main() {
    let mut ctx = Context::new();
    
    ctx.setup();
    
    let program = "5 dup * .";
    
    run_program(program, &mut ctx);
}
