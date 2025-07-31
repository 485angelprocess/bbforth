use std::{collections::HashMap, rc::Rc};

use crate::{context::{ForthRoutine, Workspace, WorkspaceContext}, types::ForthVal};

/*
Generate lazy lists
*/
#[derive(Clone, Default)]
pub struct GenEnv{
    // Variables for forms
    pub var: HashMap<String, ForthVal>,
    // Arguments from stack
    pub args: Vec<ForthVal>,
    pub counter: usize
}

/// TODO add consume method to set parameters
/// TODO int generator unit
pub trait Generator{
    fn num_args(&self) -> usize;
    fn next(&self, env: &GenEnv) -> ForthVal;
}

#[derive(Clone)]
pub struct GeneratorUnit{
    pub env: GenEnv,
    pub gen: Rc<dyn Generator>,
    pub trace: Vec<ForthVal>
}

impl GeneratorUnit{
    /// Get context from workspace
    pub fn consume(&mut self, ws: &mut WorkspaceContext){
        // TODO better error handling
        self.env.args.clear();
        for _i in 0..self.gen.num_args(){
            self.env.args.push(ws.pop().unwrap());
        }
    }
    
    /// Add operation on top of generator
    pub fn push(&mut self, v: &ForthVal) -> &mut GeneratorUnit{
        self.trace.push(v.clone());
        self
    }
    
    /// Get next value from generator
    pub fn next(&mut self) -> ForthVal{
        let result = self.gen.next(&self.env);
        
        self.env.counter += 1;
        
        if self.trace.len() > 0{
            let mut ws = Workspace::new();
            
            ws.ctx.push(result.clone());
            
            for v in &self.trace{
                ws.run(v);
            }
            return ws.ctx.pop().unwrap();
        }
        
        result
    }
}

impl PartialEq for GeneratorUnit{
    fn ne(&self, _other: &Self) -> bool {
        true
    }
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[derive(Default)]
pub struct Natural{
}

impl Generator for Natural{
    fn num_args(&self) -> usize {
        0
    }
    fn next(&self, env: &GenEnv) -> ForthVal {
        ForthVal::Float(env.counter as f64)
    }
}