pub mod mixer;

use std::sync::{Arc, Mutex};

use mixer::Mixer;

use crate::generator::GeneratorUnit;

const BUFFER_SIZE: usize = 512;

enum AudioError{
    None
}

pub struct AudioContext{
    //pub mixer: Mixer,
    gen: Vec<Arc<Mutex<GeneratorUnit>>>
}

impl AudioContext{
    pub fn new() -> Self{
        let ac = Self{
            //mixer: Mixer::new(),
            gen: Vec::new()
        };
        
        //ac.mixer.print_format();
        
        ac
    }
    
    /// push generator to audio
    pub fn push(&mut self, g: &GeneratorUnit) -> usize{
        println!("Pushing generator to audio");
        let id = self.gen.len();
        self.gen.push(Arc::new(Mutex::new(g.clone())));
        id
    }
    
    /// Audio processing main unit
    pub fn process(&mut self, id: usize, buffer: &mut [f64; BUFFER_SIZE]) -> Result<(), AudioError>{
        
        let mut channel = match self.gen.get(id){
            Some(ch) => ch.lock().unwrap(),
            None => {return Err(AudioError::None);}
        };
        
        for i in 0..buffer.len(){
            buffer[i] += channel.nextf();
        }
        
        Ok(())
    }
}