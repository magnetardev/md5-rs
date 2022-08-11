#![cfg_attr(not(test), no_std)]
mod consts;
mod util;

use consts::*;
pub use consts::{DIGEST_LEN, INPUT_BUFFER_LEN};

#[derive(Debug)]
pub struct Context {
    /// The total size of the recieved input
    pub size: u64,
    /// The input buffer
    ///
    /// Note: only access directly if you're writing to it, (e.g. if you want to write to it via Wasm memory)
    pub input: [u8; INPUT_BUFFER_LEN],
    /// The buffer for the digest
    digest: [u8; DIGEST_LEN],
    /// The working buffer
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

    /// Process the bytes in `buf`
    ///
    /// Usage:
    /// ```rs
    /// let mut ctx = Context::new();
    /// ctx.read(b"hello world");
    /// ```
    pub fn read(&mut self, buf: &[u8]) {
        let mut offset = (self.size % BLOCK_SIZE as u64) as usize;
        self.size += buf.len() as u64;
        for i in 0..buf.len() {
            self.input[offset] = buf[i];
            offset += 1;
            offset %= BLOCK_SIZE;
            if offset == 0 {
                self.step();
            }
        }
    }

    /// Process a 512-bit chunk
    pub fn step(&mut self) {
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

        // update buffer
        self.buffer[0] = self.buffer[0].wrapping_add(a);
        self.buffer[1] = self.buffer[1].wrapping_add(b);
        self.buffer[2] = self.buffer[2].wrapping_add(c);
        self.buffer[3] = self.buffer[3].wrapping_add(d);
    }

    /// Closes the reader and returns the digest
    ///
    /// Usage:
    /// ```rs
    /// let mut ctx = Context::new();
    /// ctx.read(b"hello world");
    /// let digest = ctx.finish();
    /// // prints the actual hash bytes, you need to do the hex string yourself
    /// println!("{:?}", digest);
    /// ```
    pub fn finish(mut self) -> [u8; DIGEST_LEN] {
        // Insert the padding
        let offset = (self.size % (BLOCK_SIZE as u64)) as usize;
        let padding_len: usize = if offset < 56 {
            56 - offset
        } else {
            (56 + BLOCK_SIZE) - offset
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
            assert_eq!(compute_string($input).as_str(), $hash)
        };
    }

    #[test]
    fn empty() {
        hash_eq!(b"", "d41d8cd98f00b204e9800998ecf8427e")
    }

    #[test]
    fn a() {
        hash_eq!(b"a", "0cc175b9c0f1b6a831c399e269772661")
    }

    #[test]
    fn abc() {
        hash_eq!(b"abc", "900150983cd24fb0d6963f7d28e17f72")
    }

    #[test]
    fn abcdefghijklmnopqrstuvwxyz() {
        hash_eq!(
            b"abcdefghijklmnopqrstuvwxyz",
            "c3fcd3d76192e4007dfb496cca67e13b"
        )
    }

    #[test]
    fn foo() {
        hash_eq!(b"foo", "acbd18db4cc2f85cedef654fccc4a4d8")
    }

    #[test]
    fn bar() {
        hash_eq!(b"bar", "37b51d194a7513e45b56f6524f2d51f2")
    }

    #[test]
    fn baz() {
        hash_eq!(b"baz", "73feffa4b7f6bb68e44cf984c85f6e88")
    }

    #[test]
    fn foobar() {
        hash_eq!(b"foobar", "3858f62230ac3c915f300c664312c63f")
    }

    #[test]
    fn foobarbaz() {
        hash_eq!(b"foobarbaz", "6df23dc03f9b54cc38a0fc1483df6e21")
    }

    #[test]
    fn quick_brown_fox() {
        hash_eq!(
            b"The quick brown fox jumps over the lazy dog",
            "9e107d9d372bb6826bd81d3542a419d6"
        )
    }

    #[test]
    fn hello_world() {
        hash_eq!(b"Hello, world", "bc6e6f16b8a077ef5fbc8d59d0b931b9")
    }
}
