use std::sync::{Arc, Mutex};
use cpal::{ChannelCount, SampleFormat, SampleRate, SupportedBufferSize};
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::SupportedStreamConfig;

pub type AudioBuffer = Arc<Mutex<Vec<f64>>>;

fn err_fn<T: std::fmt::Display>(err: &T){
    eprintln!("an error occurred on the output audio stream: {}", err);
}

pub struct Mixer{
    channels: Vec<AudioBuffer>,
    config: SupportedStreamConfig
}

impl Mixer{
    pub fn new() -> Self{
        
        // Setup
        let host = cpal::default_host();
        
        //for d in host.devices().unwrap(){
        //    println!("{:?}", d.name());   
        //}
        
        // TODO allow selection
        let device = host.default_output_device().expect("no output device available");
        
        println!("Default device {:?}", device.name().unwrap());
        
        let supported_configs_range = device.supported_output_configs()
                            .expect("Error while querying configs");
        
        for sc in supported_configs_range{
            
            println!("Sample format {}, Max sample rate: {}, Buffer: {:?}", 
                    sc.sample_format(),
                    sc.max_sample_rate().0,
                    sc.buffer_size()
                );
        }
        
        // Mew?
        let supported_config = SupportedStreamConfig::new(
            ChannelCount::MAX,
            SampleRate(48000),
            SupportedBufferSize::Range { min: 256, max:  1024},
            SampleFormat::F32
        );
        
        Self{
            channels: Vec::new(),
            config: supported_config
        }
    }
    
    pub fn print_format(&self){
        let sample_format = self.config.sample_format();
        //let _config = self.config.into();
        match sample_format{
            SampleFormat::F32 => println!("F32 sample format"),
            SampleFormat::I16 => println!("I16 sample format"),
            SampleFormat::U16 => println!("U16 sample format"),
            sample_format => panic!("Unsupported sample format '{sample_format}'")
        }
    }
}