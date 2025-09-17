use std::{sync::{Arc, Mutex}, time::Duration};
use std::thread;

use serialport::SerialPort;
use std::sync::MutexGuard;
use crate::types::ForthVal;


use crate::reg;
use std::fs;
use std::error::Error;

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

/// Interface for devices
pub trait DeviceInterface{
    fn unlock(&mut self) -> Result<(), String>;
    fn lock(&mut self) -> Result<(), String>;
    fn write(&mut self, addr: u32, data: u32) -> Result<usize, String>;
    fn read(&mut self, addr: u32) -> Result<i32, String>;
    
    /// Convenience obtains lock and writes
    fn single_write(&mut self, addr: u32, data: u32) -> Result<usize, String>{
        if let Ok(()) = self.lock(){
            self.write(addr, data)
        }
        else{
            Err(format!("Couldn't obtain lock"))
        }
    }
    
    /// Write data to bin
    fn to_bin(&mut self, filename: &String, start: u32, end: u32) -> Result<(), Box<dyn Error>>{
        let _result = self.lock()?;            
        let mut data = Vec::new();
        let mut addr = start;
        while addr < end{
            let result = self.read(addr)? as u32;
            
            for r in result.to_le_bytes(){
                data.push(r);
            }
            addr += 4;
        }
        self.unlock()?;
        fs::write(filename, data)?;
        Ok(())
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

impl DeviceInterface for Serial{
    fn unlock(&mut self) -> Result<(), String> {
        todo!("Implement device lock");
    }
    fn lock(&mut self) -> Result<(), String>{
        // Get device lock
        todo!("Implement device lock");
    }
    
    fn write(&mut self, addr: u32, data: u32) -> Result<usize, String>{
        // TODO wait for device to finish execution
        let mut msg = vec!['W' as u8];
        msg.append(&mut addr.to_be_bytes().to_vec());
        msg.append(&mut data.to_be_bytes().to_vec());
        
        let mut lock = self.port.lock().unwrap();
        if let Some(sp) = lock.as_mut(){
            println!("Wrote {} to addr {}", data, addr);
            Ok(sp.write(msg.as_slice()).expect("Write failed"))
        }
        else{
            println!("Serial port not open");
            Err(format!("Serial port not open"))
        }
    }
    
    fn read(&mut self, addr: u32) -> Result<i32, String>{
        let mut msg = vec!['R' as u8];
        msg.append(&mut addr.to_be_bytes().to_vec());
        
        println!("Reading from address {}", addr);
        
        let mut lock = self.port.lock().unwrap();
        if let Some(sp) = lock.as_mut(){
            let wlen = sp.write(msg.as_slice()).expect("Write failed");
            if wlen == 5{
                println!("Wrote read for address {}", addr);
                let mut serial_buf = vec![0; 32];
                match sp.read(serial_buf.as_mut_slice()){
                    Ok(_rlen) => {
                        // STUB
                        println!("Got response {:?}", serial_buf);
                        if serial_buf.len() != 9{
                            return Err(format!("Invalid response {:?}", serial_buf));
                        }
                        let mut sum:i32 = 0;
                        for i in 5..9{
                            // TODO check wrapping
                            sum = (sum << 8) + (serial_buf[i] as i32);
                        }
                        
                        return Ok(sum);
                    }
                    Err(e) => {
                        println!("Read error: {}", e);
                        return Err(format!("Read error: {}", e));
                    }
                }
            }
            else{
                println!("Request failed from serial device");
                return Err("Device responded with wrong length".to_string());
            }
        }
        else{
            println!("Serial port not open");
            return Err("Serial port not open".to_string());
        }
    }
}



// TODO fake risc device for testing
#[derive(Clone)]
pub struct RiscMock{
    memory: Arc<Mutex<Vec<u32>>>,
}

impl RiscMock{
    pub fn new() -> Self{
        Self{
            memory: Arc::new(Mutex::new(Vec::new()))
        }
    }
    
    fn read_default(lock: &MutexGuard<'_, Vec<u32>>, addr: usize, default: u32) -> u32{
        assert!(addr % 4 == 0);
        
        if let Some(value) = (*lock).get(addr >> 2){
            return value.clone();
        }
        return default;
    }
    
    fn write_expand(lock: &mut MutexGuard<'_, Vec<u32>>, addr: usize, value: u32){
        assert!(addr % 4 == 0);
        let a = addr >> 2;
        if a >= (*lock).len(){
            (*lock).resize(a+1, 0);
        }
        (*lock)[a] = value;
    }
    
    fn mutex_lock(mut lock: MutexGuard<'_, Vec<u32>>, addr: usize, set: u32) -> Result<(), ()>{
        if RiscMock::read_default(&lock, addr, 0) == 0{
            RiscMock::write_expand(&mut lock, addr, set);
            return Ok(());
        }
        Err(())
    }
}

impl DeviceInterface for RiscMock{
    fn lock(&mut self) -> Result<(), String> {
        while RiscMock::mutex_lock(self.memory.lock().unwrap(), reg::LOCK, 0b10).is_err(){
            thread::sleep(Duration::from_millis(10));
        }
        return Ok(())
    }
    fn unlock(&mut self) -> Result<(), String> {
        RiscMock::write_expand(&mut self.memory.lock().unwrap(), reg::LOCK, 0);
        Ok(())
    }
    fn read(&mut self, addr: u32) -> Result<i32, String> {
        let value = RiscMock::read_default(&self.memory.lock().unwrap(), addr as usize, 0);
        Ok(value as i32)
    }
    fn write(&mut self, addr: u32, data: u32) -> Result<usize, String> {
        RiscMock::write_expand(&mut self.memory.lock().unwrap(), addr as usize, data);
        Ok(1)
    }
}