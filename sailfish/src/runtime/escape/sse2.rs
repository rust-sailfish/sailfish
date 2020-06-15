#![allow(clippy::cast_ptr_alignment)]

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::slice;

use super::super::Buffer;
use super::naive;
use super::{ESCAPED, ESCAPED_LEN, ESCAPE_LUT};

const VECTOR_BYTES: usize = std::mem::size_of::<__m128i>();
const VECTOR_ALIGN: usize = VECTOR_BYTES - 1;

#[target_feature(enable = "sse2")]
#[inline]
pub unsafe fn escape(buffer: &mut Buffer, bytes: &[u8]) {
    let len = bytes.len();
    if len < VECTOR_BYTES {
        let start_ptr = bytes.as_ptr();
        let end_ptr = start_ptr.add(len);
        naive::escape(buffer, start_ptr, start_ptr, end_ptr);
        return;
    }

    let mut start_ptr = bytes.as_ptr();
    let end_ptr = start_ptr.add(len);

    let v_independent1 = _mm_set1_epi8(5);
    let v_independent2 = _mm_set1_epi8(2);
    let v_key1 = _mm_set1_epi8(0x27);
    let v_key2 = _mm_set1_epi8(0x3e);

    let maskgen = |x: __m128i| -> i32 {
        _mm_movemask_epi8(_mm_or_si128(
            _mm_cmpeq_epi8(_mm_or_si128(x, v_independent1), v_key1),
            _mm_cmpeq_epi8(_mm_or_si128(x, v_independent2), v_key2),
        ))
    };

    let mut ptr = start_ptr;
    let aligned_ptr = ptr.add(VECTOR_BYTES - (start_ptr as usize & VECTOR_ALIGN));

    {
        let mut mask = maskgen(_mm_loadu_si128(ptr as *const __m128i));
        loop {
            let trailing_zeros = mask.trailing_zeros() as usize;
            let ptr2 = ptr.add(trailing_zeros);
            if ptr2 >= aligned_ptr {
                break;
            }

            let c = ESCAPE_LUT[*ptr2 as usize] as usize;
            if c < ESCAPED_LEN {
                if start_ptr < ptr2 {
                    let slc = slice::from_raw_parts(
                        start_ptr,
                        ptr2 as usize - start_ptr as usize,
                    );
                    buffer.push_str(std::str::from_utf8_unchecked(slc));
                }
                buffer.push_str(*ESCAPED.get_unchecked(c));
                start_ptr = ptr2.add(1);
            }
            mask ^= 1 << trailing_zeros;
        }
    }

    ptr = aligned_ptr;
    escape_aligned(buffer, start_ptr, ptr, end_ptr);
}

#[target_feature(enable = "sse2")]
#[inline]
pub unsafe fn escape_aligned(
    buffer: &mut Buffer,
    mut start_ptr: *const u8,
    mut ptr: *const u8,
    end_ptr: *const u8,
) {
    let mut next_ptr = ptr.add(VECTOR_BYTES);
    let v_independent1 = _mm_set1_epi8(5);
    let v_independent2 = _mm_set1_epi8(2);
    let v_key1 = _mm_set1_epi8(0x27);
    let v_key2 = _mm_set1_epi8(0x3e);

    let maskgen = |x: __m128i| -> i32 {
        _mm_movemask_epi8(_mm_or_si128(
            _mm_cmpeq_epi8(_mm_or_si128(x, v_independent1), v_key1),
            _mm_cmpeq_epi8(_mm_or_si128(x, v_independent2), v_key2),
        ))
    };

    while next_ptr <= end_ptr {
        debug_assert_eq!((ptr as usize) % VECTOR_BYTES, 0);
        let mut mask = maskgen(_mm_load_si128(ptr as *const __m128i));
        while mask != 0 {
            let trailing_zeros = mask.trailing_zeros() as usize;
            let ptr2 = ptr.add(trailing_zeros);
            let c = ESCAPE_LUT[*ptr2 as usize] as usize;
            if c < ESCAPED_LEN {
                if start_ptr < ptr2 {
                    let slc = slice::from_raw_parts(
                        start_ptr,
                        ptr2 as usize - start_ptr as usize,
                    );
                    buffer.push_str(std::str::from_utf8_unchecked(slc));
                }
                buffer.push_str(*ESCAPED.get_unchecked(c));
                start_ptr = ptr2.add(1);
            }
            mask ^= 1 << trailing_zeros;
        }

        ptr = next_ptr;
        next_ptr = next_ptr.add(VECTOR_BYTES);
    }

    next_ptr = ptr.add(8);
    if next_ptr <= end_ptr {
        debug_assert_eq!((ptr as usize) % VECTOR_BYTES, 0);
        let mut mask = maskgen(_mm_loadl_epi64(ptr as *const __m128i));
        while mask != 0 {
            let trailing_zeros = mask.trailing_zeros() as usize;
            let ptr2 = ptr.add(trailing_zeros);
            let c = ESCAPE_LUT[*ptr2 as usize] as usize;
            if c < ESCAPED_LEN {
                if start_ptr < ptr2 {
                    let slc = slice::from_raw_parts(
                        start_ptr,
                        ptr2 as usize - start_ptr as usize,
                    );
                    buffer.push_str(std::str::from_utf8_unchecked(slc));
                }
                buffer.push_str(*ESCAPED.get_unchecked(c));
                start_ptr = ptr2.add(1);
            }
            mask ^= 1 << trailing_zeros;
        }

        ptr = next_ptr;
    }

    debug_assert!(ptr <= end_ptr);
    debug_assert!(start_ptr <= ptr);
    naive::escape(buffer, start_ptr, ptr, end_ptr);
}
