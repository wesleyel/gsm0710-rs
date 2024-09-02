use anyhow::Result;
use crc::Crc;

use crate::error::GsmError;

/// [Control] Field of [`Frame`]
///
/// The Control field is a 8-bit field, structured as follows:
///
/// | **Frame Type**                                 | **1** | **2** | **3** | **4** | **5** | **6** | **7** | **8** | **Notes** |
/// |------------------------------------------------|-------|-------|-------|-------|-------|-------|-------|-------|-----------|
/// | SABM (Set Asynchronous Balanced Mode)          | 1     | 1     | 1     | 1     | P/F   | 1     | 0     | 0     |           |
/// | UA (Unnumbered Acknowledgement)                | 1     | 1     | 0     | 0     | P/F   | 1     | 1     | 0     |           |
/// | DM (Disconnected Mode)                         | 1     | 1     | 1     | 1     | P/F   | 0     | 0     | 0     |           |
/// | DISC (Disconnect)                              | 1     | 1     | 0     | 0     | P/F   | 0     | 1     | 0     |           |
/// | UIH (Unnumbered Information with Header check) | 1     | 1     | 1     | 1     | P/F   | 1     | 1     | 1     |           |
/// | UI (Unnumbered Information)                    | 1     | 1     | 0     | 0     | P/F   | 0     | 0     | 0     | Optional  |
///
/// * P/F stands for Poll/Final bit.
/// * SABM (Set Asynchronous Balance Mode): SABM command shall be send by the TE (the host) to the UE (the target) to confirm the acceptance of SABM by transmission of UA response.
/// * UA (Unnumbered Acknowledgement): The UA response is sent by the module as an acknowledgement that a SABM or DISC command was accepted.
/// * DM (Disconnected Mode): In case if the module rejects SABM or DISC command, it will send DM response. For example, if SABM is sent for a DLCI not supported or if a DISC is sent to DLCI address already closed, this frame will be send.
/// * DISC (Disconnect): The DISC is used to close a previously established connection. If the application sends a DISC for the DLCI 1 and DLCI 1 is already established, then it will be closed. The module will answer to this command with an UA frame.
/// * UIH (Unnumbered Information with Header check): The UIH command/response will be used to send information. For the UIH frame, the FCS will be calculated over **only the address, control and length fields**. There is no specified response to the UIH command/response.
/// * UI (Unnumbered Information): The UI command/response will be used to send information. There is no specified response to the UI command/response. For the UI frame, the FCS shall be calculated over **all fields (Address, Control, Length Indicator, and Information)**. Support of UI frames is optional.
pub type Control = u8;
/// Address Field of [`Frame`]
///
/// <table>
///   <tr>
///     <th>Bit No.</th>
///     <td>1</td>
///     <td>2</td>
///     <td>3</td>
///     <td>4</td>
///     <td>5</td>
///     <td>6</td>
///     <td>7</td>
///     <td>8</td>
///   </tr>
///   <tr>
///     <th>Data</th>
///     <td>EA</td>
///     <td>C/R</td>
///     <td colspan=6 align="center">DLCI</td>
///   </tr>
/// </table>
///
/// * EA: Extended Address Bit. This bit is always set to 1.
/// * C/R: Command/Response Bit. See below.
/// * [`DLCI`]: Data Link Connection Identifier. This field is 6 bits long.
///
/// | Command/response | Direction              | C/R value |
/// |------------------|------------------------|-----------|
/// | Command          | Initiator -> Responder | 1         |
/// |                  | Responder -> Initiator | 0         |
/// | Response         | Initiator -> Responder | 0         |
/// |                  | Responder -> Initiator | 1         |
pub type Address = u8;

pub const FLAG: u8 = 0xF9;
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

#[allow(dead_code)]
pub trait ControlImpl {
    fn get_frame(&self) -> Result<FrameType>;
    fn set_frame(&mut self, frame: FrameType);
    fn with_frame(&self, frame: FrameType) -> Self;
    fn get_pf(&self) -> bool;
    fn set_pf(&mut self, pf: bool);
    fn with_pf(&self, pf: bool) -> Self;
    fn new_control(frame: FrameType, pf: bool) -> Self;
}

impl ControlImpl for Control {
    fn get_frame(&self) -> Result<FrameType> {
        let address = self & !PF;
        match address {
            0x2F => Ok(FrameType::SABM),
            0x63 => Ok(FrameType::UA),
            0x0F => Ok(FrameType::DM),
            0x43 => Ok(FrameType::DISC),
            0xEF => Ok(FrameType::UIH),
            0x03 => Ok(FrameType::UI),
            _ => Err(GsmError::UnsupportedFrameType(format!("{:02X?}", address)).into()),
        }
    }
    fn set_frame(&mut self, frame: FrameType) {
        let pf = *self & PF;
        let frame = match frame {
            FrameType::SABM => 0x2F,
            FrameType::UA => 0x63,
            FrameType::DM => 0x0F,
            FrameType::DISC => 0x43,
            FrameType::UIH => 0xEF,
            FrameType::UI => 0x03,
        };
        *self = frame | pf;
    }

    fn with_frame(&self, frame: FrameType) -> Self {
        let mut ctrl = *self;
        ctrl.set_frame(frame);
        ctrl
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

    fn with_pf(&self, pf: bool) -> Self {
        let mut ctrl = *self;
        ctrl.set_pf(pf);
        ctrl
    }

    fn new_control(frame: FrameType, pf: bool) -> Self {
        let mut ctrl: u8 = 0;
        ctrl.set_frame(frame);
        ctrl.set_pf(pf);
        ctrl
    }
}

#[allow(dead_code)]
pub trait AddressImpl {
    fn get_cr(&self) -> bool;
    fn set_cr(&mut self, cr: bool);
    fn with_cr(&self, cr: bool) -> Self;
    fn get_ea(&self) -> bool;
    fn set_ea(&mut self, ea: bool);
    fn with_ea(&self, ea: bool) -> Self;
    fn get_dlci(&self) -> u8;
    fn set_dlci(&mut self, dlci: u8);
    fn with_dlci(&self, dlci: u8) -> Self;
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

    fn with_cr(&self, cr: bool) -> Self {
        let mut addr = *self;
        addr.set_cr(cr);
        addr
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

    fn with_ea(&self, ea: bool) -> Self {
        let mut addr = *self;
        addr.set_ea(ea);
        addr
    }

    fn get_dlci(&self) -> u8 {
        self >> 2
    }

    fn set_dlci(&mut self, dlci: u8) {
        *self = (dlci << 2) | (*self & 0x03);
    }

    fn with_dlci(&self, dlci: u8) -> Self {
        let mut addr = *self;
        addr.set_dlci(dlci);
        addr
    }

    fn new_address(cr: bool, ea: bool, dlci: u8) -> Self {
        let mut addr: u8 = 0;
        addr.set_cr(cr);
        addr.set_ea(ea);
        addr.set_dlci(dlci);
        addr
    }
}

/// Represents a frame in the cmux protocol.
///
/// The Frame struct is defined as follows:
///
/// | **Name** | Flag    | [`Address`] | [`Control`] | Length Indicator | Information                                      | FCS     | Flag    |
/// |----------|---------|-------------|---------|------------------|--------------------------------------------------|---------|---------|
/// | **Size** | 1 octet |   1 octet   | 1 octet | 1 or 2 octets    | Unspecified length but integral number of octets | 1 octet | 1 octet |
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Frame {
    pub address: Address,
    pub control: Control,
    /// Length of the frame content. Calc by `(len & 254) >> 1`
    pub length: u16,
    pub content: Vec<u8>,
}

impl Frame {
    /// Create a new frame
    pub fn new(address: Address, control: Control, length: u16, content: Vec<u8>) -> Self {
        Frame {
            address,
            control,
            length,
            content,
        }
    }

    pub fn length_bytes(&self) -> Vec<u8> {
        let length = self.length * 2 + 1;
        if length > u8::max_value() as u16 {
            length.to_be_bytes().to_vec()
        } else {
            vec![length as u8]
        }
    }

    /// Calculate the Frame Check Sequence (FCS) of the frame
    pub fn try_fcs(&self) -> Result<u8> {
        let crc = Crc::<u8>::new(&crc::CRC_8_ROHC);
        let mut data = vec![self.address, self.control];
        data.extend_from_slice(&self.length_bytes());
        match self.control.get_frame() {
            Ok(FrameType::UI) => data.extend_from_slice(&self.content),
            Ok(_) => {}
            Err(e) => return Err(e),
        }
        // CRC-8-ROHC checksum is reversed here
        Ok(!crc.checksum(&data))
    }

    /// Parse a frame from a byte stream
    pub fn parse<T: Iterator<Item = u8>>(iter: &mut T) -> Option<(Self, usize)> {
        // 1 byte for address, 1 byte for control, 1 byte for length, 1 byte for FCS, 1 byte for flag
        let mut len = 5;
        // Find the first flag
        while let Some(byte) = iter.next() {
            len += 1;
            if byte == FLAG {
                break;
            }
        }
        // Parse the address field
        let address = iter.next()?;
        // Parse the control field
        let control = iter.next()?;
        // Parse the length field
        let mut length = iter.next()?;
        if length & 1 == 0 {
            unimplemented!("Length field is 2 octets long");
        }
        length = (length & 254) >> 1;
        // Parse the information field
        let mut content = Vec::new();
        for _ in 0..length {
            content.push(iter.next()?);
        }
        // Parse the FCS field
        let fcs = iter.next()?;
        // Parse the last flag
        let flag = iter.next()?;
        if flag != FLAG {
            return None;
        }
        len += length as usize;
        let frame = Frame {
            address,
            control,
            length: length as u16,
            content,
        };

        // validate the frame
        let fcs_calc = frame.try_fcs().ok()?;
        if fcs != fcs_calc {
            return None;
        }

        Some((frame, len))
    }

    pub fn try_to_bytes(&self) -> Result<Vec<u8>> {
        let mut data = vec![FLAG, self.address, self.control];
        if self.length > u8::max_value() as u16 {
            let len = self.length.to_be_bytes();
            data.extend_from_slice(&len);
        } else {
            data.push(((self.length as u8) << 1) | 1);
        };
        data.extend_from_slice(&self.content);
        data.push(self.try_fcs()?);
        data.push(FLAG);
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_impl_works() {
        let mut ctrl = Control::new_control(FrameType::SABM, true);
        assert_eq!(ctrl.get_frame().unwrap(), FrameType::SABM);
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

    #[test]
    fn frame_fcs_works() {
        // Frame with UI frame type
        let frame = Frame::new(7, 239, 4, vec![0x41, 0x54, 0xD, 0xA]);
        assert_eq!(frame.try_fcs().unwrap(), 0x39);
        // Frame with UIH frame type
        let addr = Address::new_address(true, true, 0x0F);
        let ctrl = Control::new_control(FrameType::UIH, true);
        let len = 0x0A;
        let frame = Frame::new(addr, ctrl, len, vec![0x41, 0x54, 0xD, 0xA]);
        assert_eq!(frame.try_fcs().unwrap(), 0x23);
    }

    #[test]
    fn frame_parse_works() {
        let frame = Frame::new(7, 239, 4, vec![0x41, 0x54, 0xD, 0xA]);
        let frame_bytes = frame.try_to_bytes().unwrap();
        dbg!(frame_bytes.clone());
        let mut iter = frame_bytes.into_iter();
        let (parsed_frame, len) = Frame::parse(&mut iter).unwrap();
        assert_eq!(parsed_frame, frame);
        assert_eq!(len, 10);
    }
}
