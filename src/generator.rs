use std::{borrow::Borrow, cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

use crate::{context::{Workspace, WorkspaceContext}, types::ForthVal};

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
    fn nextf(&mut self, env: &GenEnv) -> f64;
    fn next(&mut self, env: &GenEnv) -> ForthVal{
        ForthVal::Float(self.nextf(env))
    }
    fn make_clone(&self) -> Box<dyn Generator>;
}

pub struct GeneratorUnit{
    pub env: GenEnv,
    pub gen: Box<dyn Generator>,
    pub trace: Vec<ForthVal>,
    pub ws: Workspace
}

impl Clone for GeneratorUnit{
    fn clone(&self) -> Self {
        Self{
            env: self.env.clone(),
            trace: self.trace.clone(),
            ws: Workspace::new(),
            gen: self.gen.make_clone()
        }
    }
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
        let result = ForthVal::Float(self.nextf());
        
        result
    }
    
    pub fn nextf(&mut self) -> f64{
        let result = self.gen.nextf(&self.env);
        
        self.env.counter += 1;
        
        if self.trace.len() > 0{
            self.ws.ctx.push(ForthVal::Float(result.clone()));
            
            for v in &self.trace{
                let _ = self.ws.run(v);
            }
            return self.ws.ctx.pop().unwrap().to_float().unwrap();
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

#[derive(Default, Clone)]
pub struct Natural{
}

impl Generator for Natural{
    fn num_args(&self) -> usize {
        0
    }
    fn nextf(&mut self, env: &GenEnv) -> f64{
        env.counter as f64
    }
    fn make_clone(&self) -> Box<dyn Generator> {
        Box::new(Natural::default())
    }
}

#[derive(Default, Clone)]
pub struct Ramp{
    value: usize
}

impl Generator for Ramp{
    fn num_args(&self) -> usize {
        1
    }
    fn nextf(&mut self, env: &GenEnv) -> f64{
        let period = env.args[0].to_int().unwrap() as usize; // TODO make more flexible
        if self.value == period - 1{
            self.value = 0;
        }
        else{
            self.value += 1;
        }
        self.value as f64
    }
    fn make_clone(&self) -> Box<dyn Generator> {
        Box::new(Ramp::default())
    }
}