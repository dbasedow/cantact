#[derive(Clone,Copy,Debug)]
pub struct CanFrame {
    pub id: u32,
    pub ext: bool,
    pub rtr: bool,
    pub length: usize,
    pub data: [u8; 8],
}

impl CanFrame {
    pub fn new() -> CanFrame {
        CanFrame {
            id: 0,
            rtr: false,
            ext: false,
            length: 0,
            data: [0; 8],
        }
    }
}
