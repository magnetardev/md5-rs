# md5-rs
A simple MD5 implementation with a focus on buffered reading, and is completely `no_std`.

**This shouldn't be used in any security-critical software, as MD5 is vulnerable.**

## Motivations
I was working on a web project and realized I have no way of getting a checksum of a big file without reading it all into memory (which in some cases crashes the page!). The Web Crypto API doesn't have a MD5 implementation, so I figured rather than re-implementing a SHA algorithm, I'd implement MD5. 

As such, the API may be slightly odd since it is mainly focused on WebAssembly.

## Usage
This isn't as straight forward as other MD5 libraries may be, given that it is `no_std` through and through.
```rs
use md5_rs::Context;

// get the hash
let mut ctx = Context::new();
ctx.read(b"Hello, world");
let digest = ctx.finish();

// get digest as string
let hash = digest.iter().map(|x| format!("{:02x}", x)).collect::<String>();
println!("{hash}"); // "bc6e6f16b8a077ef5fbc8d59d0b931b9"
```
