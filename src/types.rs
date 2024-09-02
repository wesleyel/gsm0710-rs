pub type Control = u8;
const PF: u8 = 1 << 4;
const CR: u8 = 1 << 1;
const EA: u8 = 1 << 0;

#[derive(Debug, PartialEq, Eq)]
pub enum FrameType {
    SABM,
    UA,
    DM,
    DISC,
    UIH,
    UI,
}

pub trait ControlImpl {
    fn get_frame(&self) -> FrameType;
    fn is_pf(&self) -> bool;
}

impl ControlImpl for Control {
    fn get_frame(&self) -> FrameType {
        let address = self & !PF;
        match address {
            0x2F => FrameType::SABM,
            0x63 => FrameType::UA,
            0x0F => FrameType::DM,
            0x43 => FrameType::DISC,
            0xEF => FrameType::UIH,
            0x03 => FrameType::UI,
            _ => panic!("Unknown frame type"),
        }
    }
    fn is_pf(&self) -> bool {
        self & PF == PF
    }
}
