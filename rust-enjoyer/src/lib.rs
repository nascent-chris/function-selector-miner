pub mod small_string;
pub mod sponge;
#[cfg(target_feature = "avx2")]
pub mod sponges_avx;

pub use small_string::*;
pub use sponge::*;

use std::ops::{BitAnd, BitOr, BitXor, BitXorAssign, Not, Shl, Shr};

fn theta_<T>(a: &mut [T; 25], b: &[T; 5], m: usize, n: usize, o: usize)
where
    T: BitXorAssign
        + BitXor<Output = T>
        + Shl<u32, Output = T>
        + Shr<u32, Output = T>
        + BitOr<Output = T>
        + Copy,
{
    let t = b[m] ^ rotate_left(b[n], 1);
    a[o] ^= t;
    a[o + 5] ^= t;
    a[o + 10] ^= t;
    a[o + 15] ^= t;
    a[o + 20] ^= t;
}

pub fn theta<T>(a: &mut [T; 25], b: &mut [T; 5])
where
    T: BitXorAssign
        + BitXor<Output = T>
        + Shl<u32, Output = T>
        + Shr<u32, Output = T>
        + BitOr<Output = T>
        + Copy,
{
    b[0] = a[0] ^ a[5] ^ a[10] ^ a[15] ^ a[20];
    b[1] = a[1] ^ a[6] ^ a[11] ^ a[16] ^ a[21];
    b[2] = a[2] ^ a[7] ^ a[12] ^ a[17] ^ a[22];
    b[3] = a[3] ^ a[8] ^ a[13] ^ a[18] ^ a[23];
    b[4] = a[4] ^ a[9] ^ a[14] ^ a[19] ^ a[24];

    theta_(a, b, 4, 1, 0);
    theta_(a, b, 0, 2, 1);
    theta_(a, b, 1, 3, 2);
    theta_(a, b, 2, 4, 3);
    theta_(a, b, 3, 0, 4);
}

fn rho_pi<T>(a: &mut [T; 25], b: &mut [T; 5])
where
    T: Copy + Shl<u32, Output = T> + Shr<u32, Output = T> + BitOr<Output = T>,
{
    let t = a[1];
    b[0] = a[10];
    a[10] = rotate_left(t, 1);
    rho_pi_(a, b, 7, 3);
    rho_pi_(a, b, 11, 6);
    rho_pi_(a, b, 17, 10);
    rho_pi_(a, b, 18, 15);
    rho_pi_(a, b, 3, 21);
    rho_pi_(a, b, 5, 28);
    rho_pi_(a, b, 16, 36);
    rho_pi_(a, b, 8, 45);
    rho_pi_(a, b, 21, 55);
    rho_pi_(a, b, 24, 2);
    rho_pi_(a, b, 4, 14);
    rho_pi_(a, b, 15, 27);
    rho_pi_(a, b, 23, 41);
    rho_pi_(a, b, 19, 56);
    rho_pi_(a, b, 13, 8);
    rho_pi_(a, b, 12, 25);
    rho_pi_(a, b, 2, 43);
    rho_pi_(a, b, 20, 62);
    rho_pi_(a, b, 14, 18);
    rho_pi_(a, b, 22, 39);
    rho_pi_(a, b, 9, 61);
    rho_pi_(a, b, 6, 20);
    rho_pi_(a, b, 1, 44);
}

fn rho_pi_<T>(a: &mut [T; 25], b: &mut [T; 5], m: usize, n: u32)
where
    T: Copy + Shl<u32, Output = T> + Shr<u32, Output = T> + BitOr<Output = T>,
{
    let t = b[0];
    b[0] = a[m];
    a[m] = rotate_left(t, n);
}

fn rotate_left<T>(value: T, shift: u32) -> T
where
    T: Copy + Shl<u32, Output = T> + Shr<u32, Output = T> + BitOr<Output = T>,
{
    (value << shift) | (value >> (64 - shift))
}

fn chi<T>(a: &mut [T; 25])
where
    T: Not<Output = T> + BitAnd<Output = T> + BitXor<Output = T> + Default + Copy,
{
    let mut b = [T::default(); 5];
    chi_(a, &mut b, 0);
    chi_(a, &mut b, 5);
    chi_(a, &mut b, 10);
    chi_(a, &mut b, 15);
    chi_(a, &mut b, 20);
}

fn chi_<T>(a: &mut [T], b: &mut [T], n: usize)
where
    T: Not<Output = T> + BitAnd<Output = T> + BitXor<Output = T> + Copy,
{
    b[0] = a[n];
    b[1] = a[n + 1];
    b[2] = a[n + 2];
    b[3] = a[n + 3];
    b[4] = a[n + 4];
    a[n] = b[0] ^ ((!b[1]) & b[2]);
    a[n + 1] = b[1] ^ ((!b[2]) & b[3]);
    a[n + 2] = b[2] ^ ((!b[3]) & b[4]);
    a[n + 3] = b[3] ^ ((!b[4]) & b[0]);
    a[n + 4] = b[4] ^ ((!b[0]) & b[1]);
}

fn iota<T, U>(a: &mut [T], x: U)
where
    T: BitXorAssign<U> + Copy,
{
    a[0] ^= x;
}

pub const fn normalize_endianess(x: u32) -> u32 {
    x.to_be()
}

pub fn function_selector_to_hex(x: u32) -> String {
    format!("0x{x:0x}")
}
