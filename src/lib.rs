#![cfg_attr(not(test), no_std)]
mod consts;
mod util;

pub use consts::*;

#[derive(Debug)]
pub struct Context {
    size: u64,
    pub input: [u8; INPUT_BUFFER_LEN],
    digest: [u8; DIGEST_LEN],
    buffer: [u32; 4],
}

impl Context {
    pub fn new() -> Self {
        Self {
            size: 0,
            input: [0; INPUT_BUFFER_LEN],
            digest: [0; DIGEST_LEN],
            buffer: [A, B, C, D],
        }
    }

    /// Process the bytes in the input buffer
    ///
    /// Note: this should only be used if you're writing to the Context.input array directly (e.g. if you're writing to it via Wasm memory).
    /// Otherwise, use `Context::read`, but do note that it clones the data.
    pub fn update(&mut self, bytes_written: usize) {
        self.size += bytes_written as u64;
        if self.size % 64 != 0 {
            return;
        }
        self.step();
    }

    pub fn read(&mut self, buf: &[u8]) {
        let mut offset = (self.size % 64) as usize;
        self.size += buf.len() as u64;
        for i in 0..buf.len() {
            self.input[offset] = buf[i];
            offset += 1;

            if offset % 64 == 0 {
                self.input.copy_from_slice(buf);
                self.step();
            }
        }
    }

    /// Process a 512-bit chunk
    fn step(&mut self) {
        let [mut a, mut b, mut c, mut d] = self.buffer;
        let mut e: u32;
        let mut g: usize;
        for i in 0..BLOCK_SIZE {
            if i < 16 {
                e = util::f(b, c, d);
                g = i;
            } else if i < 32 {
                e = util::g(b, c, d);
                g = ((i * 5) + 1) % 16;
            } else if i < 48 {
                e = util::h(b, c, d);
                g = ((i * 3) + 5) % 16;
            } else {
                e = util::i(b, c, d);
                g = (i * 7) % 16;
            }
            g *= 4;

            // get a u32 from input at index g
            let mut u32_input: u32 = 0;
            u32_input |= (self.input[g + 3] as u32) << 24;
            u32_input |= (self.input[g + 2] as u32) << 16;
            u32_input |= (self.input[g + 1] as u32) << 8;
            u32_input |= self.input[g] as u32;

            let f = a.wrapping_add(e).wrapping_add(K[i]).wrapping_add(u32_input);
            a = d;
            d = c;
            c = b;
            b = b.wrapping_add(util::rotate_u32_left(f, S[i]));
        }
        self.buffer[0] = self.buffer[0].wrapping_add(a);
        self.buffer[1] = self.buffer[1].wrapping_add(b);
        self.buffer[2] = self.buffer[2].wrapping_add(c);
        self.buffer[3] = self.buffer[3].wrapping_add(d);
    }

    /// Closes the reader and returns the digest
    pub fn finish(mut self) -> [u8; DIGEST_LEN] {
        // Insert the padding
        let offset = (self.size % 64) as usize;
        let padding_len: usize = if offset < 56 {
            56 - offset
        } else {
            (56 + 64) - offset
        };
        self.read(&PADDING[..padding_len]);
        self.size -= padding_len as u64;

        // Do a final update
        self.input[(INPUT_BUFFER_LEN - 8)..]
            .copy_from_slice((self.size * 8).to_ne_bytes().as_slice());
        self.step();

        // Finalize the digest
        for i in 0..4 {
            self.digest[i * 4] = (self.buffer[i] & 0x000000FF) as u8;
            self.digest[(i * 4) + 1] = ((self.buffer[i] & 0x0000FF00) >> 8) as u8;
            self.digest[(i * 4) + 2] = ((self.buffer[i] & 0x00FF0000) >> 16) as u8;
            self.digest[(i * 4) + 3] = ((self.buffer[i] & 0xFF000000) >> 24) as u8;
        }
        self.digest
    }
}

#[cfg(test)]
mod test {
    use super::Context;

    fn compute_string(bytes: &[u8]) -> String {
        let mut ctx = Context::new();
        ctx.read(bytes);
        let digest = ctx.finish();
        digest
            .iter()
            .map(|x| format!("{:02x}", x))
            .collect::<String>()
    }

    macro_rules! hash_eq {
        ($input:expr, $hash:expr) => {
            assert_eq!(compute_string($input).as_str(), $hash);
        };
    }

    #[test]
    fn empty() {
        hash_eq!(b"", "d41d8cd98f00b204e9800998ecf8427e");
    }

    #[test]
    fn quick_brown_fox() {
        hash_eq!(
            b"The quick brown fox jumps over the lazy dog",
            "9e107d9d372bb6826bd81d3542a419d6"
        );
    }
}
