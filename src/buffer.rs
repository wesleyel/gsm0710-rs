use ringbuffer::RingBuffer;

use crate::types::{Frame, FLAG};

pub const GSM0710_BUFFER_CAPACITY: usize = 2048;

pub trait GSM0710Buffer {
    fn push_vec(&mut self, vec: Vec<u8>);
    /// Pop a GSM 07.10 frame from the buffer
    ///
    /// If a frame is found, it is returned Some(Frame)
    /// If no frame is found, None is returned.
    fn pop_frame(&mut self) -> Option<Frame>;
    /// Pop at least one frame from the buffer.
    ///
    /// If a frame is found, it is returned Some(Frame)
    /// If buffer is empty, None is returned.
    fn pop_frame1(&mut self) -> Option<Frame>;
}

impl<T: RingBuffer<u8>> GSM0710Buffer for T {
    fn push_vec(&mut self, vec: Vec<u8>) {
        for byte in vec {
            self.push(byte);
        }
    }

    fn pop_frame(&mut self) -> Option<Frame> {
        let buf = self.to_vec();
        match Frame::parse(&mut buf.into_iter()) {
            Some((frame, len)) => {
                for _ in 0..len {
                    self.skip();
                }
                Some(frame)
            }
            None => {
                // Discard all bytes until the next FLAG
                loop {
                    if let Some(byte) = self.dequeue() {
                        if byte == FLAG {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                None
            }
        }
    }

    fn pop_frame1(&mut self) -> Option<Frame> {
        if self.is_empty() {
            return None;
        }
        if self.to_vec().iter().find(|&&b| b == FLAG).is_none() {
            self.clear();
            return None;
        }
        loop {
            let frame = self.pop_frame();
            if frame.is_some() {
                return frame;
            } else {
                return self.pop_frame1();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ringbuffer::AllocRingBuffer;

    #[test]
    fn ringbuffer_works() {
        let mut buffer = AllocRingBuffer::<u8>::new(GSM0710_BUFFER_CAPACITY);
        assert_eq!(buffer.capacity(), GSM0710_BUFFER_CAPACITY);
        buffer.push(0x01);
        buffer.push(0x02);
        assert_eq!(buffer.len(), 2);
    }

    #[test]
    fn gsm0710_buffer_push_vec() {
        let mut buffer = AllocRingBuffer::<u8>::new(GSM0710_BUFFER_CAPACITY);
        let vec = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        buffer.push_vec(vec.clone());
        assert_eq!(buffer.len(), 5);
        assert_eq!(buffer.to_vec(), vec)
    }

    #[test]
    fn gsm0710_buffer_pop_frame_multiple_frames() {
        let mut buffer = AllocRingBuffer::<u8>::new(GSM0710_BUFFER_CAPACITY);
        let frame1 = Frame::new(7, 239, 4, vec![0x41, 0x54, 0xD, 0xA]);
        let frame2 = Frame::new(13, 239, 4, vec![0x44, 0x55, 0xD, 0xA]);
        let frame1_bytes = frame1.try_to_bytes().unwrap();
        let frame2_bytes = frame2.try_to_bytes().unwrap();
        buffer.push_vec(frame1_bytes.clone());
        // Push an extra FLAG as garbage bytes
        buffer.push(FLAG);
        buffer.push_vec(frame2_bytes.clone());
        let popped_frame1 = buffer.pop_frame();
        let popped_frame2 = buffer.pop_frame();
        let popped_frame3 = buffer.pop_frame();
        assert_eq!(popped_frame1, Some(frame1));
        assert_eq!(popped_frame2, None);
        assert_eq!(popped_frame3, Some(frame2));
    }

    #[test]
    fn gsm0710_buffer_pop_frame_no_frame() {
        let mut buffer = AllocRingBuffer::<u8>::new(GSM0710_BUFFER_CAPACITY);
        let vec = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        buffer.push_vec(vec.clone());
        let popped_frame = buffer.pop_frame();
        assert_eq!(popped_frame, None);
    }

    #[test]
    fn gsm0710_buffer_pop_frame1() {
        let mut buffer = AllocRingBuffer::<u8>::new(GSM0710_BUFFER_CAPACITY);
        let frame1 = Frame::new(7, 239, 4, vec![0x41, 0x54, 0xD, 0xA]);
        let frame2 = Frame::new(13, 239, 4, vec![0x44, 0x55, 0xD, 0xA]);
        let frame1_bytes = frame1.try_to_bytes().unwrap();
        let frame2_bytes = frame2.try_to_bytes().unwrap();
        buffer.push_vec(frame1_bytes.clone());
        // Push an extra FLAG as garbage bytes
        buffer.push(FLAG);
        buffer.push_vec(frame2_bytes.clone());
        // frame1 is popped first
        let popped_frame1 = buffer.pop_frame1();
        // frame2 is popped next. Cause pop_frame1 will discard the garbage bytes
        let popped_frame2 = buffer.pop_frame1();
        // No frame is found
        let popped_frame3 = buffer.pop_frame1();
        assert_eq!(popped_frame1, Some(frame1));
        assert_eq!(popped_frame2, Some(frame2));
        assert_eq!(popped_frame3, None);
    }
}
