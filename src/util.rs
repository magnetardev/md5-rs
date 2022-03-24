use core::ops::{BitAnd, BitOr, BitXor, Not};

#[inline]
pub fn rotate_u32_left(x: u32, n: u32) -> u32 {
    (x << n) | (x >> (32 - n))
}

#[inline]
pub fn f<T>(x: T, y: T, z: T) -> T
where
    T: Copy + Not<Output = T> + BitAnd<Output = T> + BitOr<Output = T>,
{
    (x & y) | (!x & z)
}

#[inline]
pub fn g<T>(x: T, y: T, z: T) -> T
where
    T: Copy + Not<Output = T> + BitAnd<Output = T> + BitOr<Output = T>,
{
    (x & z) | (y & !z)
}

#[inline]
pub fn h<T>(x: T, y: T, z: T) -> T
where
    T: Copy + BitXor<Output = T>,
{
    x ^ y ^ z
}

#[inline]
pub fn i<T>(x: T, y: T, z: T) -> T
where
    T: Copy + Not<Output = T> + BitOr<Output = T> + BitXor<Output = T>,
{
    y ^ (x | !z)
}
