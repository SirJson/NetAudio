use byteorder::{ByteOrder, NetworkEndian};
use rodio::Sink;
use cpal::SampleFormat;
use rodio::{Sample, Source, source::UniformSourceIterator};
use std::collections::VecDeque;
use std::net::UdpSocket;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use std::convert::TryInto;

type SampleType = f32;
const UDP_BUFFER_SIZE: usize = 65507;
const NETWORK_ADDRESS: &str = "0.0.0.0:11331";

struct AudioStream {
    data: VecDeque<SampleType>,
    receiver: Receiver<Vec<SampleType>>,
}

impl AudioStream {
    fn new(receiver: Receiver<Vec<SampleType>>) -> Self {
        AudioStream {
            data: VecDeque::new(),
            receiver,
        }
    }
}

impl Iterator for AudioStream {
    type Item = SampleType;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(foo) = self.receiver.try_recv() {
            self.data.extend(&foo);
        }
        let out = self.data.pop_front();
        if out == None {
            Some(Self::Item::zero_value())
        } else {
            out
        }
    }
}

impl Source for AudioStream {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        48000
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

fn main() -> std::io::Result<()> {
    let key = "DEBUG";
    let debug = match std::env::var(key) {
        Ok(val) => val == "1",
        Err(_) => false,
    };
    if debug {
        println!("Debug mode!");
    }
    println!("NetAudio Server v1.0");
    let (tx, rx): (Sender<Vec<SampleType>>, Receiver<Vec<SampleType>>) = mpsc::channel();
    let mut buffer = Box::new([0; UDP_BUFFER_SIZE]);

    let device = rodio::default_output_device().expect("Failed to select default output device");
    println!("Output Device: {}",device.name());
    let device_format = device.default_output_format().expect("No default output format!");
    println!("Default format: {:?}", device_format);
    let supported_formats = device.supported_output_formats().expect("No supported output formats!");
    for format in supported_formats {
        println!("\tSupported format: {:?}", format);
    }

    let sink = Sink::new(&device);
    let source_stream = AudioStream::new(rx);
    let stream = UniformSourceIterator::<AudioStream,SampleType>::new(source_stream, device_format.channels, device_format.sample_rate.0);
    println!("Binding to address: {}",NETWORK_ADDRESS);
    let socket = UdpSocket::bind(NETWORK_ADDRESS).expect("Failed to bind network address");
    
    match device_format.data_type {
        SampleFormat::F32 => sink.append(stream.convert_samples::<f32>()),
        SampleFormat::I16 => sink.append(stream.convert_samples::<i16>()),
        SampleFormat::U16 => sink.append(stream.convert_samples::<u16>())
    }
    sink.play();

    let mut now = std::time::Instant::now();

    println!("Starting main loop!");
    loop {
        let bytes_recv = match socket.recv_from(&mut *buffer) {
            Ok(n) => n.0,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => 0,
            Err(e) => panic!("Encountered Network IO error: {}", e),
        };
        if bytes_recv > 0 {
            if debug && now.elapsed().as_secs() > 5 {
                println!("Bytes received: {}",bytes_recv);
                now = std::time::Instant::now();
            }
            let source = &mut buffer[..bytes_recv];
            if bytes_recv % 4 != 0 {
                eprintln!("Bytes received is not a multiple of 4! Skipping packet...");
                continue;
            }
            let mut target: Vec<f32> = vec![0.0; bytes_recv / 4];
            NetworkEndian::read_f32_into(&source, &mut target);
            if let Err(e) = tx.send(target) {
                panic!("Internal error: {}", e);
            }
        }
    }

    Ok(())
}
