extern crate libc;
extern crate nix;

use nix::sys::termios::{tcgetattr, cfmakeraw, ControlFlags, InputFlags, SetArg, tcsetattr, cfsetspeed, BaudRate};

use std::env::args;
use std::fs::{OpenOptions};
use std::io::{Write, BufReader, BufRead};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;

fn main() {
    let device = args().last().unwrap();
    println!("opening {:?}", device);
    let mut options = OpenOptions::new();
    options.write(true);
    options.read(true);
    if cfg!(unix) {
        options.custom_flags(libc::O_NOCTTY);
        options.custom_flags(libc::O_NONBLOCK);
    }

    let mut file = options.open(device).unwrap();
    let mut tios = tcgetattr(file.as_raw_fd()).unwrap();
    cfmakeraw(&mut tios);
    tios.input_flags &= !InputFlags::IXOFF;
    tios.control_flags &= !ControlFlags::CRTSCTS;
    cfsetspeed(&mut tios, BaudRate::B115200).expect("cfsetspeed failed");

    tcsetattr(file.as_raw_fd(), SetArg::TCSADRAIN, &tios).expect("tcsetattr failed");

    file.write(b"C\rS6\rO\r");

    let mut rdr = BufReader::new(file);
    let mut line = String::new();
    rdr.read_line(&mut line);
}
