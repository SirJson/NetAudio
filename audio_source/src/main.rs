use byteorder::{ByteOrder, NetworkEndian};
use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    println!("Connecting..");
    let socket = UdpSocket::bind("0.0.0.0:0").expect("couldn't bind to address");
    socket
        .connect("localhost:11331")
        .expect("connect function failed");

    println!("Start stream..");

    // Use the default host for working with audio devices.
    let host = cpal::default_host();

    // Setup the default input device and stream with the default input format.
    let device = host
        .default_output_device()
        .expect("Failed to get default output device");
    println!("Default input device: {}", device.name().unwrap());
    let format = device
        .default_output_format()
        .expect("Failed to get default output format");
    println!("Default input format: {:?}", format);
    let event_loop = host.event_loop();
    let stream_id = event_loop.build_input_stream(&device, &format).unwrap();
    event_loop.play_stream(stream_id).unwrap();
    event_loop.run(move |id, event| {
        let data = match event {
            Ok(data) => data,
            Err(err) => {
                eprintln!("an error occurred on stream {:?}: {}", id, err);
                return;
            }
        };
        // Otherwise write to the wav writer.
        match data {
            cpal::StreamData::Input {
                buffer: cpal::UnknownTypeInputBuffer::U16(buffer),
            } => {
                let mut pkg = vec![0; buffer.len() * 2];
                NetworkEndian::write_u16_into(&buffer, pkg.as_mut_slice());
                socket.send(&pkg).expect("Failed to send package");
            }
            cpal::StreamData::Input {
                buffer: cpal::UnknownTypeInputBuffer::I16(buffer),
            } => {
                let mut pkg = vec![0; buffer.len() * 2];
                NetworkEndian::write_i16_into(&buffer, pkg.as_mut_slice());
                socket.send(&pkg).expect("Failed to send package");
            }
            cpal::StreamData::Input {
                buffer: cpal::UnknownTypeInputBuffer::F32(buffer),
            } => {
                let mut pkg = vec![0; buffer.len() * 4];
                NetworkEndian::write_f32_into(&buffer, pkg.as_mut_slice());
                socket.send(&pkg).expect("Failed to send package");
            }
            _ => (),
        }
    });
}