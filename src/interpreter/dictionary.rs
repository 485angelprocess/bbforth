use std::{collections::HashMap, rc::Rc};

use super::{generator, ForthFn, ForthFnGen, ForthRoutine, Generator};

#[derive(Clone)]
pub struct Dictionary{
    // Main library
    lookup: HashMap<String, usize>,
    library: HashMap<usize, ForthRoutine>,

    local_lookup: HashMap<String, usize>,
    local_library: HashMap<usize, ForthRoutine>,
    
    local: bool
}

impl Dictionary{
    pub fn new() -> Self{
        Self{
            lookup: HashMap::new(),
            library: HashMap::new(),
            
            local_lookup: HashMap::new(),
            local_library: HashMap::new(),
            
            local: false
        }
    }
    
    /// Set context
    pub fn set_context(&mut self, local: bool){
        self.local = local;
    }
    
    pub fn is_local(&self) -> bool{
        self.local
    }
    
    /// Insert new definition
    pub fn insert(&mut self, s: &str, f: ForthFn) -> usize{
        self.insert_routine(&s.to_string(), ForthRoutine::Prim(Rc::new(f)))
    }
    
    /// Insert function pointer
    pub fn insert_ptr(&mut self, s: &str, f: ForthFnGen) -> usize{
        self.insert_routine(&s.to_string(), ForthRoutine::Prim(f))
    }
    
    /// Insert generator object
    pub fn insert_generator<T: Generator + Default + 'static>(&mut self, s: &str) -> usize{
        self.insert_routine(&s.to_string(), ForthRoutine::Prim(Rc::new(|ws| generator::<T>(ws))))
    }
    
    /// Insert routine
    pub fn insert_routine(&mut self, s: &String, f: ForthRoutine) -> usize{
        
        let (lookup, library) = match self.local{
                false => (&mut self.lookup, &mut self.library),
                true => (&mut self.local_lookup, &mut self.local_library)
        };
        
        let id = match lookup.get(s){
            Some(v) => v.clone(),
            None => lookup.len()
        };
        
        lookup.insert(s.clone(), id);
        library.insert(id, f);
        id
    }
    
    /// Special control lookup
    pub fn then_id(&self) -> usize{
        *self.get_id("then").unwrap()
    }
    
    /// Get byte code for a function string
    pub fn get_id(&self, s: &str) -> Option<&usize>{
        if self.local{
            if let Some(id) = self.local_lookup.get(s){
                return Some(id);
            }
        }
        self.lookup.get(s)
    }
    
    /// Get function from string name
    pub fn get_fn(&self, s: &str) -> Option<&ForthRoutine>{
        if let Some(id) = self.get_id(s){
            self.get_fn_from_id(id)
        }
        else{
            None
        }
    }
    
    /// Get function from id code
    pub fn get_fn_from_id(&self, v: &usize) -> Option<&ForthRoutine>{
        if self.local{
            if let Some(f) = self.local_library.get(v){
                return Some(f);
            }
        }
        self.library.get(v)
    }
}