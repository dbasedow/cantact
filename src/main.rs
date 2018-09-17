extern crate serial;

use serial::prelude::*;
use std::env::args;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::time::Duration;

fn main() {
    let device = args().last().unwrap();
    println!("opening {}", device);
    let mut port = serial::open(&device).unwrap();
    interact(&mut port).unwrap();
}

fn interact<T: SerialPort>(port: &mut T) -> io::Result<()> {
    println!("trying to reconfigure");
    try!(port.reconfigure(&|settings| {
        try!(settings.set_baud_rate(serial::Baud115200));
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    }));

    println!("reconfiguration successful");

    try!(port.set_timeout(Duration::from_millis(1000)));

    println!("trying to write");

    try!(port.write(b"C\r"));
    try!(port.write(b"S6\r"));
    try!(port.write(b"O\r"));

    let mut frame_buf = [0; 40];
    let mut buf = [0; 1];
    let mut i = 0;

    //let mut reader = BufReader::new(port);
    loop {
        match port.read_exact(&mut buf) {
            Ok(()) if buf[0] == b'\r' => {
                //println!("{:?}", &frame_buf[0..i]);
                let frame = slcan::parse_serial_line(&frame_buf[0..i]);
                i = 0;
            }
            Ok(()) => {
                frame_buf[i] = buf[0];
                i += 1;
            }
            Err(e) => println!("{}", e),
        }
    }
}

mod canframe;
mod slcan;
mod protocols;
