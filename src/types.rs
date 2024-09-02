pub type Control = u8;
pub type Address = u8;

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
    fn set_frame(&mut self, frame: FrameType);
    fn get_pf(&self) -> bool;
    fn set_pf(&mut self, pf: bool);
    fn new_control(frame: FrameType, pf: bool) -> Self;
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
    fn set_frame(&mut self, frame: FrameType) {
        let frame = match frame {
            FrameType::SABM => 0x2F,
            FrameType::UA => 0x63,
            FrameType::DM => 0x0F,
            FrameType::DISC => 0x43,
            FrameType::UIH => 0xEF,
            FrameType::UI => 0x03,
        };
        *self = frame;
    }

    fn get_pf(&self) -> bool {
        self & PF == PF
    }

    fn set_pf(&mut self, pf: bool) {
        *self = match pf {
            true => *self | PF,
            false => *self & !PF,
        };
    }

    fn new_control(frame: FrameType, pf: bool) -> Self {
        let mut ctrl: u8 = 0;
        ctrl.set_frame(frame);
        ctrl.set_pf(pf);
        ctrl
    }
}

pub trait AddressImpl {
    fn get_cr(&self) -> bool;
    fn set_cr(&mut self, cr: bool);
    fn get_ea(&self) -> bool;
    fn set_ea(&mut self, ea: bool);
    fn get_dlci(&self) -> u8;
    fn set_dlci(&mut self, dlci: u8);
    fn new_address(cr: bool, ea: bool, dlci: u8) -> Self;
}

impl AddressImpl for Address {
    fn get_cr(&self) -> bool {
        self & CR == CR
    }

    fn set_cr(&mut self, cr: bool) {
        *self = match cr {
            true => *self | CR,
            false => *self & !CR,
        };
    }

    fn get_ea(&self) -> bool {
        self & EA == EA
    }

    fn set_ea(&mut self, ea: bool) {
        *self = match ea {
            true => *self | EA,
            false => *self & !EA,
        };
    }

    fn get_dlci(&self) -> u8 {
        self >> 2
    }

    fn set_dlci(&mut self, dlci: u8) {
        *self = (dlci << 2) | (*self & 0x03);
    }

    fn new_address(cr: bool, ea: bool, dlci: u8) -> Self {
        let mut addr: u8 = 0;
        addr.set_cr(cr);
        addr.set_ea(ea);
        addr.set_dlci(dlci);
        addr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_impl_works() {
        let mut ctrl = Control::new_control(FrameType::SABM, true);
        assert_eq!(ctrl.get_frame(), FrameType::SABM);
        assert_eq!(ctrl.get_pf(), true);
        ctrl.set_frame(FrameType::UA);
        ctrl.set_pf(false);
        assert_eq!(ctrl, 0x63);
    }

    #[test]
    fn address_impl_works() {
        let mut addr = Address::new_address(true, true, 0x0F);
        assert_eq!(addr.get_cr(), true);
        assert_eq!(addr.get_ea(), true);
        assert_eq!(addr.get_dlci(), 0x0F);
        addr.set_cr(false);
        addr.set_ea(false);
        addr.set_dlci(0x03);
        assert_eq!(addr, 0x03 << 2);
    }
}
