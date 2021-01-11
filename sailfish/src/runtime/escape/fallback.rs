#![allow(clippy::cast_ptr_alignment)]

use super::super::Buffer;
use super::naive;

#[cfg(target_pointer_width = "16")]
const USIZE_BYTES: usize = 2;

#[cfg(target_pointer_width = "32")]
const USIZE_BYTES: usize = 4;

#[cfg(target_pointer_width = "64")]
const USIZE_BYTES: usize = 8;

const USIZE_ALIGN: usize = USIZE_BYTES - 1;

#[inline]
fn contains_zero_byte(x: usize) -> bool {
    const LO_U64: u64 = 0x0101_0101_0101_0101;
    const HI_U64: u64 = 0x8080_8080_8080_8080;
    const LO_USIZE: usize = LO_U64 as usize;
    const HI_USIZE: usize = HI_U64 as usize;

    x.wrapping_sub(LO_USIZE) & !x & HI_USIZE != 0
}

#[inline]
fn contains_key(x: usize) -> bool {
    const INDEPENDENTS1: usize = 0x0505_0505_0505_0505_u64 as usize;
    const INDEPENDENTS2: usize = 0x0202_0202_0202_0202_u64 as usize;
    const KEY1: usize = 0x2727_2727_2727_2727_u64 as usize;
    const KEY2: usize = 0x3e3e_3e3e_3e3e_3e3e_u64 as usize;

    let y1 = x | INDEPENDENTS1;
    let y2 = x | INDEPENDENTS2;
    let z1 = y1 ^ KEY1;
    let z2 = y2 ^ KEY2;
    contains_zero_byte(z1) || contains_zero_byte(z2)
}

#[inline]
pub unsafe fn escape(feed: &str, buffer: &mut Buffer) {
    debug_assert!(feed.len() >= 16);

    let len = feed.len();
    let mut start_ptr = feed.as_ptr();
    let end_ptr = feed[len..].as_ptr();

    let mut ptr = start_ptr;
    let aligned_ptr = ptr.add(USIZE_BYTES - (start_ptr as usize & USIZE_ALIGN));
    debug_assert_eq!(aligned_ptr as usize % USIZE_BYTES, 0);
    debug_assert!(aligned_ptr <= end_ptr);

    let chunk = (ptr as *const usize).read_unaligned();
    if contains_key(chunk) {
        start_ptr = naive::proceed(buffer, start_ptr, ptr, aligned_ptr);
    }

    ptr = aligned_ptr;

    while ptr <= end_ptr.sub(USIZE_BYTES) {
        debug_assert_eq!((ptr as usize) % USIZE_BYTES, 0);

        let chunk = *(ptr as *const usize);
        if contains_key(chunk) {
            start_ptr = naive::proceed(buffer, start_ptr, ptr, ptr.add(USIZE_BYTES))
        }
        ptr = ptr.add(USIZE_BYTES);
    }
    debug_assert!(ptr <= end_ptr);
    debug_assert!(start_ptr <= ptr);
    naive::escape(buffer, start_ptr, ptr, end_ptr);
}
