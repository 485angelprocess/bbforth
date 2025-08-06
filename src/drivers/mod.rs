use std::time::Duration;

use std::rc::Rc;
use serialport::SerialPort;

use crate::types::ForthVal;

fn to_bytes(v: &ForthVal) -> Vec<u8>{
   match v{
       ForthVal::Str(s) => s.as_bytes().to_vec(),
       ForthVal::Int(v) => (*v as u32).to_be_bytes().to_vec(),
       _ => panic!("Can't convert {:?} to bytes", v)
   }
}

/*
External serial device
*/
pub struct Serial{
    port: Option<Box<dyn SerialPort>>
}

impl Serial{
    pub fn new() -> Self{
        Self{
            port: None
        }
    }
    
    pub fn print_ports() -> ForthVal{
        let mut ports = Vec::new();
        
        for p in serialport::available_ports().expect("No ports found"){
            println!("{}", p.port_name);
            ports.push(ForthVal::Str(p.port_name));
        }
        
        ForthVal::List(ports)
    }
    
    pub fn start(&mut self, p: &ForthVal, b: &ForthVal) -> ForthVal{
        let baud = b.to_int().unwrap() as u32;
        match p{
            ForthVal::Str(s) => {
                println!("Connecting to serial port {} with baud {}", s, baud);
                let s = serialport::new(s.as_str(), baud) 
                    .timeout(Duration::from_millis(100))
                    .open();
                self.port = match s{
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
        println!("Sending bytes {:?}", bytes);
        
        if let Some(p) = &mut self.port{
            let wlen = p.write(bytes.as_slice()).expect("Write failed");
            println!("Wrote {} bytes", wlen);
        }
        else{
            // temp tell me the info
            return ForthVal::Err(format!("Port not open"));
        }
        return ForthVal::Null;
    }
    
    pub fn get(&mut self) -> ForthVal{
        if let Some(p) = &mut self.port{
            // TODO make this is more a gets n bytes
            let mut serial_buf = vec![0; 32];
            match p.read(serial_buf.as_mut_slice()){
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