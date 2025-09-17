/* Virtual memory with arguments names */
// TODO have this map to both local and client memory correctly
use std::collections::HashMap;
use crate::types::{ForthVal, AsmPromise};
use crate::drivers::DeviceInterface;
use crate::asm;

use std::error::Error;

use std::rc::Rc;
use std::cell::RefCell;

use crate::reg;

#[derive(Clone)]
pub enum Location{
    Local(usize),
    Client(u32, u32)
}

pub struct VariableMemory{
    names: HashMap<String, Location>,
    local: Vec<ForthVal>,
    driver: Rc<RefCell<dyn DeviceInterface>>,
    promises: HashMap<String, (u32, AsmPromise)>,
    heapreg: u32
}



impl VariableMemory{
    pub fn new(d: Rc<RefCell<dyn DeviceInterface>>, start: u32) -> Self{
        Self{
            names: HashMap::new(),
            local: Vec::new(),
            driver: d,
            promises: HashMap::new(),
            heapreg: start
        }
    }
    
    fn write_to_mem(&mut self, addr: u32, offset: u32, value: &ForthVal) -> Result<u32, Box<dyn Error>>{
        let hp = addr+offset;
        match value{
            ForthVal::Int(v) => {
                self.driver.borrow_mut().write(hp, *v as u32)?;
                Ok(1)
            },
            ForthVal::List(vals) => {
                let mut written = 0;
                for v in vals{
                    let size = self.write_to_mem(addr+(written << 2), offset, v)?;
                    written += size;
                }
                Ok(vals.len() as u32)
            },
            ForthVal::Promise((name, p)) => {
                self.add_promise(name, hp, p);
                Ok(1)
            },
            ForthVal::Meta(label) => {
                println!("Label registered at location {}", hp);
                self.names.insert(label.clone(), Location::Client(hp, 1));
                Ok(0)
            },
            _ => todo!("Invalid type")
        }
    }
    
    pub fn access_local(&self, v: usize) -> Option<&ForthVal>{
        self.local.get(v)
    }
    
    pub fn access_client(&self, addr: usize) -> Result<ForthVal, ForthVal>{
        let mut d = self.driver.borrow_mut();
        if let Ok(_) = d.lock(){
            let result = d.read((addr+reg::OFFSET) as u32);
            let _result = d.unlock();
            match result{
                Ok(v) => Ok(ForthVal::Int(v as i64)),
                Err(s) => Err(ForthVal::Err(s))
            }
        }
        else{
            Err(ForthVal::Err("Couldn't get lock".to_string()))
        }
    }
    
    pub fn get(&mut self, s: &String) -> Option<ForthVal>{
        self.names.get(s).map(|loc|{ForthVal::Var(loc.clone())})
    }
    
    pub fn assign_local(&mut self, name: &String, value: &ForthVal){
        // TODO check for already named value
        let loc = match self.names.get(name){
            Some(Location::Local(v)) => *v,
            Some(Location::Client(_, _)) => panic!("Client var already exists with that name"),
            None => self.local.len() 
        };
        self.local.push(value.clone());
        self.names.insert(name.clone(), Location::Local(loc));
    }
    
    pub fn assign_client(&mut self, name: &String, value: &ForthVal) -> Result<(), String>{
        let lock = self.driver.borrow_mut().lock();
        if let Ok(_) = lock{
            // Get serial lock
            let heapaddr = {
                let heapaddr = self.driver.borrow_mut().read(self.heapreg).expect("Read heap register") as usize;
                heapaddr as u32
            };
            
            let offset = reg::OFFSET as u32;
            
            let hp= heapaddr+offset as u32;
            
            println!("Here is {} (offset address {})", heapaddr, hp);
            
            let size: u32 = self.write_to_mem(heapaddr, offset, value).unwrap();
            
            // Save address
            self.names.insert(name.clone(), Location::Client(heapaddr, size));
            
            let regaddr = self.heapreg;
            // Increment variable space
            self.driver.borrow_mut().write(regaddr, heapaddr+(size<<2))?;
            let _result = self.driver.borrow_mut().unlock();
            
            self.run_promise();
            
            return Ok(());
        }
        Err(format!("Couldnt get driver lock"))
    }
    
    /// Convert to binary file
    /// Useful for running virtual memory on qemu
    pub fn to_bin(&mut self, filename: &String) -> Result<(), Box<dyn Error>>{
        let heapaddr = self.driver.borrow_mut().read(self.heapreg)? as u32;
        
        println!("Here is {}", heapaddr);
        
        self.driver.borrow_mut().to_bin(filename, reg::OFFSET as u32, heapaddr+(reg::OFFSET as u32))?;
        Ok(())
    }
    
    pub fn run_promise(&mut self){
        for (name, (addr, promise)) in &self.promises{
            if let Some(Location::Client(addrset, _size)) = self.names.get(name){
                match promise{
                    AsmPromise::JAL(rd) => {
                        // TODO try to not to redo if it's already set
                        let offset = (*addrset as i32) - (*addr as i32);
                        println!("Promise on {} at location {} has jal to addr {} offset {}", name, addr, addrset, offset);
                        let jal = asm::jal(*rd, offset as u32);
                        println!("JAL {:#02X}", jal);
                        let _result = self.driver.borrow_mut().write(*addr, jal);
                    }
                }
            }
        }
    }
    
    pub fn add_promise(&mut self, name: &String, addr: u32, promise: &AsmPromise){
        self.promises.insert(name.clone(), (addr, promise.clone()));
    }
    
    fn client_values_above(&mut self, base_address: u32) -> Vec<&String>{
        let mut vals = Vec::new();
        for (name, loc) in &self.names{
            match loc{
                Location::Client(addr, _size) => {
                    if *addr > base_address{
                        vals.push(name)
                    }
                },
                _ => ()
            }
        }
        vals
    }
    
    pub fn dealloc_client(&mut self, name: &String) -> Result<(), String>{
        // Copy values above the deallocated unit
        // This ideally is fast work :)
        // Could make this a dedicated function
        if let Some(&Location::Client(base_address, _size)) = self.names.get(name){
            for var in self.client_values_above(base_address){
                println!("Copying {}", var);
            }
        }
        todo!("Deallocatate")
    }
}