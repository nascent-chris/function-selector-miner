use crate::SmallString;

use std::str;

pub struct Sponge {
    pub uint64s: [u64; 25],
}

impl Default for Sponge {
    fn default() -> Self {
        Self {
            uint64s: [0u64; 25],
        }
    }
}

impl Sponge {
    pub fn set_byte(&mut self, o: usize, value: u8) {
        let uint64_index = o / 8;
        let byte_index = o % 8;

        // Clear the target byte
        self.uint64s[uint64_index] &= !(0xFFu64 << (byte_index * 8));
        // Set the target byte
        self.uint64s[uint64_index] |= (value as u64) << (byte_index * 8);
    }

    pub fn fill(&mut self, function_name: &SmallString, nonce: u64, function_params: &SmallString) {
        let o = self.fill_sponge(function_name, nonce, function_params);
        // self.chars[o] = 0x01;
        self.set_byte(o, 0x01);
    }

    pub fn fill_zero(&mut self, offset: usize, end: usize) {
        let start_index = offset / 8;
        let end_index = (end + 7) / 8; // Ensure covering the last byte

        for i in start_index..end_index {
            if i * 8 < offset || (i + 1) * 8 > end {
                // Partially filled u64 elements
                for j in 0..8 {
                    let byte_pos = i * 8 + j;
                    if byte_pos >= offset && byte_pos < end {
                        unsafe {
                            // Zero out each byte individually
                            *(&mut self.uint64s[i] as *mut u64 as *mut u8).add(j) = 0;
                        }
                    }
                }
            } else {
                // Fully covered u64 elements
                self.uint64s[i] = 0;
            }
        }
    }

    pub fn to_string(&self, o: usize) -> String {
        let mut bytes = Vec::new();
        for i in 0..((o + 7) / 8) {
            let uint64 = self.uint64s[i];
            for j in 0..8 {
                if i * 8 + j >= o {
                    break;
                }
                bytes.push((uint64 >> (j * 8)) as u8);
            }
        }

        unsafe { str::from_utf8_unchecked(&bytes) }.to_owned()
    }

    pub fn fill_and_get_name(
        &mut self,
        function_name: &SmallString,
        nonce: u64,
        function_params: &SmallString,
    ) -> String {
        let o = self.fill_sponge(function_name, nonce, function_params);
        self.set_byte(o, 0x00);

        // str::from_utf8_unchecked(&self.chars[..o]).to_owned()
        self.to_string(o)
    }

    pub fn write_decimal(&mut self, offset: usize, x: u64) -> usize {
        let mut buff = [0u8; 64];
        let mid = 32;
        let mut p = mid;

        let mut x_mut = x;
        while x_mut != 0 {
            p -= 1;
            buff[p] = (x_mut % 10) as u8 + b'0';
            x_mut /= 10;
        }

        let len = mid - p;
        self.copy_bytes_to_uint64s(offset, &buff[p..mid]);
        len
    }

    fn copy_bytes_to_uint64s(&mut self, mut offset: usize, bytes: &[u8]) {
        for &byte in bytes {
            let uint64_index = offset / 8;
            let byte_index = offset % 8;
            unsafe {
                *(&mut self.uint64s[uint64_index] as *mut u64 as *mut u8).add(byte_index) = byte;
            }
            offset += 1;
        }
    }

    /// # Safety
    ///
    /// This function is unsafe because it writes to a union type.
    pub fn fill_sponge(
        &mut self,
        function_name: &SmallString,
        nonce: u64,
        function_params: &SmallString,
    ) -> usize {
        let mut offset = self.fill_sponge_single(0, function_name);
        // offset += write_decimal(&mut self.chars[offset..], nonce);
        offset += self.write_decimal(offset, nonce);
        offset += self.fill_sponge_single(offset, function_params);

        let end = 200;
        // self.chars[offset..end].fill(0);
        self.fill_zero(offset, end);

        // self.chars[135] = 0x80;
        self.set_byte(135, 0x80);
        offset
    }

    pub fn copy_from_slice(&mut self, offset: usize, s: &[u8]) {
        let mut byte_pos = offset;
        for &byte in s {
            let uint64_index = byte_pos / 8;
            let byte_index = byte_pos % 8;
            unsafe {
                *(&mut self.uint64s[uint64_index] as *mut u64 as *mut u8).add(byte_index) = byte;
            }
            byte_pos += 1;
        }
    }

    /// # Safety
    ///
    /// This function is unsafe because it writes to a union type.
    pub fn fill_sponge_single(&mut self, offset: usize, s: &SmallString) -> usize {
        // self.chars[offset..][..s.length].copy_from_slice(&s.data[..s.length]);
        self.copy_from_slice(offset, &s.data[..s.length]);

        s.length
    }

    /// # Safety
    ///
    /// This function is unsafe because it uses SIMD instructions and a union type.
    pub fn compute_selectors(&mut self) -> u32 {
        crate::iters(&mut self.uint64s);
        self.uint64s[0] as u32
    }
}

// fn write_decimal(out: &mut [u8], mut x: u64) -> usize {
//     let mut buff = [0u8; 64];
//     let mid = 32;
//     let mut p = mid;

//     while x != 0 {
//         p -= 1;
//         buff[p] = (x % 10) as u8 + b'0';
//         x /= 10;
//     }

//     let len = mid - p;
//     out[..len].copy_from_slice(&buff[p..mid]);
//     len
// }
