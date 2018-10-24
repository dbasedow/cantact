use canframe::CanFrame;
use std::io::Write;
use std::io;
use bytes::BytesMut;
use std::str;
use tokio::codec::{Decoder, Encoder, Framed};

pub struct CanFrameCodec;

impl Decoder for CanFrameCodec {
    type Item = CanFrame;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let newline = src.as_ref().iter().position(|b| *b == b'\r');
        if let Some(n) = newline {
            let line = src.split_to(n + 1);
            return match parse_serial_line(&line) {
                Ok(frame) => Ok(Some(frame)),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid String")),
            };
        }
        Ok(None)
    }
}

impl Encoder for CanFrameCodec {
    type Item = CanFrame;
    type Error = io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let line = frame_to_serial_line(item);
        dst.extend_from_slice(&line);

        Ok(())
    }
}

pub fn parse_serial_line(line: &[u8]) -> Result<CanFrame, String> {
    let mut frame = CanFrame::new();
    frame.ext = match line[0] {
        b'T' | b'R' => true,
        _ => false,
    };
    frame.rtr = match line[0] {
        b'r' | b'R' => true,
        _ => false,
    };

    let data_offset;

    if !frame.ext {
        frame.id = hex_to_u32(&line[1..4]).unwrap();
        frame.length = hex_digit_to_int(line[4]) as usize;
        data_offset = 5;
    } else {
        frame.id = hex_to_u32(&line[1..9]).unwrap();
        frame.length = hex_digit_to_int(line[9]) as usize;
        data_offset = 10;
    }

    if frame.length > 0 {
        let str_len = frame.length * 2;
        let d = &line[data_offset..data_offset + str_len];
        let mut i = 0;
        let it = d
            .chunks(2)
            .map(|b| hex_digit_to_int(b[0]) << 4 | hex_digit_to_int(b[1]));
        for b in it {
            frame.data[i] = b;
            i += 1;
        }
    }

    Ok(frame)
}

pub fn frame_to_serial_line(frame: CanFrame) -> Vec<u8> {
    let mut line = vec![0; 30];
    match frame {
        CanFrame { ext, rtr, .. } if ext && rtr => line[0] = b'R',
        CanFrame { ext, rtr, .. } if !ext && rtr => line[0] = b'r',
        CanFrame { ext, rtr, .. } if ext && !rtr => line[0] = b'T',
        CanFrame { ext, rtr, .. } if !ext && !rtr => line[0] = b't',
        _ => unreachable!(),
    }

    u32_to_hex_canid(frame.id, &mut line[1..]);

    let mut offset = if frame.ext { 9 } else { 4 };
    write!(&mut line[offset..], "{:X}", frame.length);
    offset += 1;
    for b in frame.data[..frame.length].iter() {
        write!(&mut line[offset..], "{:02X}", b);
        offset += 2;
    }
    line[offset] = b'\r';
    line.truncate(offset + 1);

    line
}

fn hex_to_u32(input: &[u8]) -> Option<u32> {
    if input.len() < 1 || input.len() > 8 {
        return None;
    }

    let mut result: u32 = 0;
    for (i, hex_nible) in input.iter().rev().enumerate() {
        result += match *hex_nible {
            b'0'...b'9' => ((hex_nible - b'0') as u32) << i * 4,
            b'A'...b'F' => ((hex_nible - b'A') as u32) << i * 4,
            b'a'...b'f' => ((hex_nible - b'a') as u32) << i * 4,
            _ => return None,
        }
    }
    Some(result)
}

fn u32_to_hex_canid(num: u32, buf: &mut [u8]) {
    if num <= 0x800 {
        write!(&mut buf[..], "{:03X}", num);
    } else {
        write!(&mut buf[..], "{:08X}", num);
    }
}

#[test]
fn test_u32_to_hex_canid() {
    let mut data = vec![0; 10];

    u32_to_hex_canid(255, &mut data[..]);
    assert_eq!(str::from_utf8(&data[0..3]).unwrap(), "0FF");

    u32_to_hex_canid(0x01, &mut data[..]);
    assert_eq!(str::from_utf8(&data[0..3]).unwrap(), "001");

    u32_to_hex_canid(0x20000000, &mut data[..]);
    assert_eq!(str::from_utf8(&data[0..8]).unwrap(), "20000000");

    u32_to_hex_canid(0x10000, &mut data[..]);
    assert_eq!(str::from_utf8(&data[0..8]).unwrap(), "00010000");
}

fn hex_digit_to_int(hex: u8) -> u8 {
    match hex {
        b'0'...b'9' => hex & 0x0f,
        b'a'...b'f' | b'A'...b'F' => (hex & 0x0f) + 9,
        _ => panic!("non ascii character {}", hex),
    }
}

#[test]
fn test_hex_digit_to_int() {
    assert_eq!(hex_digit_to_int(b'0'), 0);
    assert_eq!(hex_digit_to_int(b'9'), 9);
    assert_eq!(hex_digit_to_int(b'a'), 10);
    assert_eq!(hex_digit_to_int(b'A'), 10);
    assert_eq!(hex_digit_to_int(b'f'), 15);
    assert_eq!(hex_digit_to_int(b'F'), 15);
}

#[test]
fn test_parse_slcan_frames() {
    let frame = parse_serial_line(b"T12345678100").unwrap();
    assert_eq!(frame.id, 0x12345678);
    assert_eq!(frame.length, 1);
    assert_eq!(frame.data[0], 0);
    assert!(frame.ext);
    assert!(!frame.rtr);

    let frame = parse_serial_line(b"R123456780").unwrap();
    assert_eq!(frame.id, 0x12345678);
    assert_eq!(frame.length, 0);
    assert!(frame.ext);
    assert!(frame.rtr);

    let frame = parse_serial_line(b"t1230").unwrap();
    assert!(!frame.ext);
    assert!(!frame.rtr);
    assert_eq!(frame.id, 0x123);
    assert_eq!(frame.length, 0);

    let frame = parse_serial_line(b"t1231aF").unwrap();
    assert!(!frame.ext);
    assert!(!frame.rtr);
    assert_eq!(frame.id, 0x123);
    assert_eq!(frame.length, 1);
    assert_eq!(frame.data[0], 0xaf);

    let frame = parse_serial_line(b"r1230").unwrap();
    assert!(!frame.ext);
    assert!(frame.rtr);
}

#[test]
fn test_frame_to_serial_line() {
    let frame = CanFrame {
        id: 0x12345678,
        length: 1,
        data: [0; 8],
        ext: true,
        rtr: false,
    };

    let line = frame_to_serial_line(frame);
    assert_eq!(line, b"T12345678100\r");

    let frame = CanFrame {
        id: 0x12345678,
        ext: true,
        rtr: true,
        length: 0,
        data: [0; 8],
    };
    let line = frame_to_serial_line(frame);
    assert_eq!(line, b"R123456780\r");

    let frame = CanFrame {
        id: 0x123,
        ext: false,
        rtr: false,
        length: 0,
        data: [0; 8],
    };
    let line = frame_to_serial_line(frame);
    assert_eq!(line, b"t1230\r");

    let frame = CanFrame {
        id: 0x123,
        ext: false,
        rtr: false,
        length: 1,
        data: [0xaf, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    };
    let line = frame_to_serial_line(frame);
    assert_eq!(line, b"t1231AF\r");

    let frame = CanFrame {
        id: 0x123,
        ext: false,
        rtr: true,
        length: 0,
        data: [0; 8],
    };
    let line = frame_to_serial_line(frame);
    assert_eq!(line, b"r1230\r");
}
