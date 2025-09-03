/* Virtual memory with arguments names */
// TODO have this map to both local and client memory correctly
use std::collections::HashMap;
use crate::types::ForthVal;

enum Location{
    Local(usize),
    Client(usize)
}

struct VariableMemory{
    names: HashMap<String, Location>,
    local: Vec<ForthVal>,
}

impl VariableMemory{
    pub fn new() -> Self{
        Self{
            names: HashMap::new(),
            local: Vec::new()
        }
    }
    
    pub fn assign_local(&mut self, name: &String, value: &ForthVal){
        let loc = self.local.len();
        self.local.push(value.clone());
        self.names.insert(name.clone(), Location::Local(loc));
    }
}