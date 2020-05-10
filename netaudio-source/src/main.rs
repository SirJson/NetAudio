use byteorder::{ByteOrder, NetworkEndian};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use num_traits::Num;


fn print_usage(program: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();

    let mut opts = getopts::Options::new();
    opts.optopt("t", "target", "the target audio server", "IP");
    opts.optopt("p", "port", "the target port", "PORT");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("d", "debug", "debug mode");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return Ok(())
    }

    let debug_mode = matches.opt_present("d");

    let ip = match matches.opt_str("t") {
        Some(i) => i,
        None => "localhost".to_owned()
    };

    let port = match matches.opt_str("p") {
        Some(i) => i,
        None => "11331".to_owned()
    };

    let netaddr = format!("{}:{}",ip,port);

    println!("Connecting..");

    let socket = UdpSocket::bind("0.0.0.0:0").expect("couldn't bind to address");
    socket
        .connect(netaddr)
        .expect("connect function failed");
    let socket = Arc::new(Mutex::new(socket));
    let work_socket = socket.clone();

    println!("Start stream..");

    // Use the default host for working with audio devices.
    let host = cpal::default_host();

    // Setup the default input device and stream with the default input format.
    let device = host
        .default_output_device()
        .expect("Failed to get default output device");
    println!("Default output device: {}", device.name().unwrap());

    let config = device.default_input_config().unwrap();
    println!("Default input config: {:?}", config);

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(&config.into(),  move |data, _: &_| write_input_data::<f32, f32>(data, &work_socket), err_fn)?,
        cpal::SampleFormat::I16 => device.build_input_stream(&config.into(),  move |data, _: &_| write_input_data::<i16, i16>(data, &work_socket), err_fn)?,
        cpal::SampleFormat::U16 => device.build_input_stream(&config.into(),  move |data, _: &_| write_input_data::<u16, u16>(data, &work_socket), err_fn)?,
    };
}

fn write_input_data<T,U>(input: &[T], socket: &Arc<Mutex<UdpSocket>>)
where
    T: cpal::Sample,
    U: cpal::Sample + Num
{
    if let Ok(mut socket) = socket.try_lock() {
            for &sample in input.iter() {
                let sample: U = cpal::Sample::from(&sample);
                match(sample) {

                }
            }
        }

}