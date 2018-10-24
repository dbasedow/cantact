extern crate bytes;
#[macro_use]
extern crate futures;
extern crate serial;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_serial;
extern crate mio_serial;

use canframe::CanFrame;
use std::{env, io, str};
use tokio::codec::{Decoder, Encoder, Framed};
use tokio_serial::SerialPort;
use slcan::CanFrameCodec;
use bytes::BytesMut;

use futures::{Future, Sink, Stream};

use std::env::args;

struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let newline = src.as_ref().iter().position(|b| *b == b'\r');
        if let Some(n) = newline {
            let line = src.split_to(n + 1);
            return match str::from_utf8(line.as_ref()) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid String")),
            };
        }
        Ok(None)
    }
}

impl Encoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn encode(&mut self, _item: Self::Item, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn main() {
    let device = args().last().unwrap();
    println!("opening {}", device);

    let settings = tokio_serial::SerialPortSettings::default();
    let mut port = tokio_serial::Serial::from_path(device, &settings).unwrap();
    port.set_baud_rate(tokio_serial::BaudRate::Baud115200)
        .expect("unable to set baud rate");

    #[cfg(unix)]
    port.set_exclusive(false)
        .expect("Unable to set serial port exclusiveness");

    let fut = tokio_io::io::write_all(port, b"C\rS6\rO\r")
        .and_then(|(port, _)| {
            let (writer, reader) = Framed::new(port, CanFrameCodec{}).split();
            println!("wrote stuff");
            let printer = reader
                .for_each(|s| {
                    println!("{:?}", s);
                    Ok(())
                }).map_err(|e| eprintln!("{}", e));
            tokio::spawn(printer);

            let w = writer
                .send(CanFrame {
                    id: 0x713,
                    ext: false,
                    rtr: false,
                    length: 8,
                    data: [0x03, 0x22, 0xf1, 0x91, 0x55, 0x55, 0x55, 0x55],
                }).and_then(|_| {
                    println!("wrote more stuff");
                    Ok(())
                }).map_err(|e| eprintln!("{}", e));

            tokio::spawn(w);

            Ok(())
        }).map_err(|e| eprintln!("{}", e));

    tokio::run(fut);
}

mod canframe;
mod protocols;
mod slcan;
