use std::{sync::{Arc, Mutex}, time::Duration};

use serialport::SerialPort;

use crate::types::ForthVal;

/// Convert forth value into bytes
fn to_bytes(v: &ForthVal) -> Vec<u8>{
   match v{
       ForthVal::Str(s) => s.as_bytes().to_vec(),
       ForthVal::Int(v) => (*v as u32).to_be_bytes().to_vec(),
       ForthVal::List(mlist) => {
        let mut result = Vec::new();
        for lv in mlist{
            result.append(&mut to_bytes(lv));
        }
        result
       },
       _ => panic!("Can't convert {:?} to bytes", v)
   }
}

/// Serial port driver
#[derive(Clone)]
pub struct Serial{
    port: Arc<Mutex<Option<Box<dyn SerialPort>>>>
    
    // TODO set verbose flag or have history buffer
}

impl Serial{
    pub fn new() -> Self{
        Self{
            port: Arc::new(Mutex::new(None))
        }
    }
    
    pub fn print_ports() -> ForthVal{
        let mut ports = Vec::new();
        
        for p in serialport::available_ports().expect("No ports found"){
            //println!("{}", p.port_name);
            ports.push(ForthVal::Str(p.port_name));
        }
        
        ForthVal::List(ports)
    }
    
    pub fn available(&self) -> bool{
        self.port.lock().unwrap().is_some()
    }
    
    pub fn start(&mut self, p: &ForthVal, b: &ForthVal) -> ForthVal{
        let baud = b.to_int().unwrap() as u32;
        match p{
            ForthVal::Str(s) => {
                println!("Connecting to serial port {} with baud {}", s, baud);
                let s = serialport::new(s.as_str(), baud) 
                    .timeout(Duration::from_millis(100))
                    .open();
                let mut sp = self.port.lock().unwrap();
                *sp = match s{
                    Ok(p) => Some(p),
                    Err(e) => {return ForthVal::Err(format!("{:?}", e));}
                }
            }
            _ => {
                return ForthVal::Err(format!("Unable to open {p:?}"));
            }
        }
        ForthVal::Null
    }
    
    pub fn put(&mut self, msg: &ForthVal) -> ForthVal{
        let bytes = to_bytes(msg);
        
        if self.available(){
            let mut sp = self.port.lock().unwrap();
            let _wlen = (sp.as_mut())
                    .expect("Serial port exists")
                    .write(bytes.as_slice()).expect("Write failed");
        }
        else{
            // temp tell me the info
            return ForthVal::Err(format!("Port not open"));
        }
        return ForthVal::Null;
    }
    
    pub fn write(&mut self, addr: u32, data: u32) -> Option<usize>{
        // TODO wait for device to finish execution
        let mut msg = vec!['W' as u8];
        msg.append(&mut addr.to_be_bytes().to_vec());
        msg.append(&mut data.to_be_bytes().to_vec());
        
        let mut lock = self.port.lock().unwrap();
        if let Some(sp) = lock.as_mut(){
            println!("Wrote {} to addr {}", data, addr);
            Some(sp.write(msg.as_slice()).expect("Write failed"))
        }
        else{
            println!("Serial port not open");
            None
        }
    }
    
    pub fn read(&mut self, addr: u32) -> Option<u32>{
        let mut msg = vec!['R' as u8];
        msg.append(&mut addr.to_be_bytes().to_vec());
        
        let mut lock = self.port.lock().unwrap();
        if let Some(sp) = lock.as_mut(){
            println!("Reading from address {}", addr);
            let wlen = sp.write(msg.as_slice()).expect("Write failed");
            if wlen == 5{
                println!("Wrote read for address {}", addr);
                let mut serial_buf = vec![0; 32];
                match sp.read(serial_buf.as_mut_slice()){
                    Ok(_rlen) => {
                        // STUB
                        println!("Got response {:?}", serial_buf);
                    }
                    Err(e) => {
                        println!("Read error: {}", e);
                    }
                }
            }
            else{
                println!("Request failed from serial device");
            }
            // Failed
            None
        }
        else{
            println!("Serial port not open");
            None
        }
    }
    
    pub fn get(&mut self) -> ForthVal{
        if self.available(){
            let mut lock = self.port.lock().unwrap();
            
            // TODO make this is more a gets n bytes
            let mut serial_buf = vec![0; 32];
            match lock.as_mut().unwrap().read(serial_buf.as_mut_slice()){
                Ok(rlen) => {
                    let mut resp = Vec::new();
                    for b in &mut serial_buf[0..rlen]{
                        resp.push(ForthVal::Int(*b as i64));
                    }
                    return ForthVal::List(resp);
                },
                Err(e) => {
                    return ForthVal::Err(format!("{:?}", e));
                }
            }
        }
        else{
            return ForthVal::Err(format!("Port not open"));
        }
    }
}