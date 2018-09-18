use canframe::CanFrame;

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
