extern crate bytes;
extern crate futures;
extern crate serial;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_serial;

use std::{env, io, str};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_io::codec::{Decoder, Encoder};
use tokio_serial::SerialPort;

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
        .expect("Unable to set serial port exlusive");

    println!("reading");

    let fut = tokio_io::io::write_all(port, b"C\rS6\rO\r")
        .and_then(|(port, _)| {
            let (writer, reader) = port.framed(LineCodec).split();
            println!("wrote stuff");
            let printer = reader
                .for_each(|s| {
                    println!("{:?}", s);
                    Ok(())
                }).map_err(|e| eprintln!("{}", e));
            tokio::spawn(printer);

            let w = writer
                .send("t200720c00010040301\r".to_string())
                .and_then(|_| {
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
