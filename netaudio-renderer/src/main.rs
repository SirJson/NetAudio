use byteorder::{ByteOrder, NetworkEndian};
use rodio::Sink;
use rodio::{Sample, Source, source::UniformSourceIterator};
use std::collections::VecDeque;
use std::net::UdpSocket;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use cpal::traits::DeviceTrait;
use cpal::SampleFormat;
use getopts;

type SampleType = f32;
const UDP_BUFFER_SIZE: usize = 65507;

#[derive(Debug)]
struct Config {
    ip: Option<String>,
    port: Option<u32>
}

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
        if let Ok(sample) = self.receiver.try_recv() {
            self.data.extend(&sample);
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


fn print_usage(program: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}


fn print_capabilities() {
    let device = rodio::default_output_device().expect("Failed to select default output device");
    println!("Output Device: {:?}",device.name());
    let device_format = device.default_output_format().expect("No default output format!");
    println!("Default format: {:?}", device_format);
    let supported_formats = device.supported_output_formats().expect("No supported output formats!");
    for format in supported_formats {
        println!("\tSupported format: {:?}", format);
    }
}

fn main() -> std::io::Result<()> {
    let debug = match std::env::var("DEBUG") {
        Ok(val) => val == "1",
        Err(_) => false,
    };
    if debug {
        println!("Debug mode!");
    }

    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();

    let mut opts = getopts::Options::new();
    opts.optopt("i", "ip", "ip the server will bind to", "IP");
    opts.optopt("p", "port", "port the server will bind to", "PORT");
    opts.optopt("s", "samplerate", "specifies the output sample rate if supported", "SAMPLERATE");
    opts.optflag("c", "capabilities", "prints a list of all possible formats of the default device");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return Ok(())
    }

    if matches.opt_present("c") {
        print_capabilities();
        return Ok(())
    }

    let ip = match matches.opt_str("i") {
        Some(i) => i,
        None => "0.0.0.0".to_owned()
    };

    let port = match matches.opt_str("p") {
        Some(i) => i,
        None => "11331".to_owned()
    };

    let netaddr = format!("{}:{}",ip,port);

    println!("NetAudio Server v1.0");
    let (tx, rx): (Sender<Vec<SampleType>>, Receiver<Vec<SampleType>>) = mpsc::channel();
    let mut buffer = Box::new([0; UDP_BUFFER_SIZE]);

    let device = rodio::default_output_device().expect("Failed to select default output device");
    println!("Output Device: {:?}",device.name());

    let device_format = match matches.opt_str("s") {
        Some(s) => {
            let mut supported_formats = device.supported_output_formats().expect("No supported output formats!");
            match supported_formats.find(|f| f.max_sample_rate.0 == s.parse::<u32>().expect("Specified sample rate is not an integer")) {
                Some(f) => f.with_max_sample_rate(),
                None => device.default_output_format().expect("No default output format!")
            }
        },
        None => device.default_output_format().expect("No default output format!")
    };


    println!("Output format: {:?}", device_format);

    let sink = Sink::new(&device);
    let source_stream = AudioStream::new(rx);
    let stream = UniformSourceIterator::<AudioStream,SampleType>::new(source_stream, device_format.channels, device_format.sample_rate.0);
    println!("Binding to address: {}",netaddr);
    let socket = UdpSocket::bind(netaddr).expect("Failed to bind network address");
    
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
}
