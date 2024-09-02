use ringbuffer::RingBuffer;

const GSM0710_BUFFER_CAPACITY: usize = 2048;

pub trait GSM0710Buffer {
    fn push_vec(&mut self, vec: Vec<u8>);
    fn pop_frame(&mut self) -> Option<Vec<u8>>;
}

impl<T: RingBuffer<u8>> GSM0710Buffer for T {
    fn push_vec(&mut self, vec: Vec<u8>) {
        for byte in vec {
            self.push(byte);
        }
    }

    fn pop_frame(&mut self) -> Option<Vec<u8>> {
        // skip everything until we find the FCS
        loop {
            let byte = match self.peek() {
                Some(byte) => byte,
                None => return None,
            };

            if *byte != 0xF9 {
                self.skip();
            } else {
                break;
            }
        }
        // find the next FCS and collect everything in between
        match self.to_vec().iter().skip(1).position(|&x| x == 0xF9) {
            Some(idx) => {
                let collected: Vec<u8> = self.to_vec().drain(0..=idx + 1).collect();
                for _ in 0..=idx + 1 {
                    self.skip();
                }
                return Some(collected);
            }
            None => return None,
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
}
