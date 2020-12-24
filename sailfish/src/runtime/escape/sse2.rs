#![allow(clippy::cast_ptr_alignment)]

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::slice;

use super::super::Buffer;
use super::naive::push_escaped_str;
use super::{ESCAPED, ESCAPED_LEN, ESCAPE_LUT};

const VECTOR_BYTES: usize = std::mem::size_of::<__m128i>();

#[target_feature(enable = "sse2")]
pub unsafe fn escape(feed: &str, buffer: &mut Buffer) {
    let len = feed.len();
    let mut start_ptr = feed.as_ptr();
    let mut ptr = start_ptr;
    let end_ptr = feed[len..].as_ptr();

    let v_independent1 = _mm_set1_epi8(5);
    let v_independent2 = _mm_set1_epi8(2);
    let v_key1 = _mm_set1_epi8(0x27);
    let v_key2 = _mm_set1_epi8(0x3e);

    let maskgen = |x: __m128i| -> u32 {
        _mm_movemask_epi8(_mm_or_si128(
            _mm_cmpeq_epi8(_mm_or_si128(x, v_independent1), v_key1),
            _mm_cmpeq_epi8(_mm_or_si128(x, v_independent2), v_key2),
        )) as u32
    };

    while ptr <= end_ptr.sub(VECTOR_BYTES) {
        let mut mask = maskgen(_mm_loadu_si128(ptr as *const __m128i));
        while mask != 0 {
            let trailing_zeros = mask.trailing_zeros() as usize;
            mask ^= 1 << trailing_zeros;
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
                push_escaped_str(*ESCAPED.get_unchecked(c), buffer);
                start_ptr = ptr2.add(1);
            }
        }

        ptr = ptr.add(VECTOR_BYTES);
    }

    debug_assert!(ptr.add(VECTOR_BYTES) > end_ptr);

    if ptr < end_ptr {
        debug_assert!((end_ptr as usize - ptr as usize) < VECTOR_BYTES);
        let backs = VECTOR_BYTES - (end_ptr as usize - ptr as usize);
        let read_ptr = ptr.sub(backs);

        let mut mask = maskgen(_mm_loadu_si128(read_ptr as *const __m128i)) >> backs;
        while mask != 0 {
            let trailing_zeros = mask.trailing_zeros() as usize;
            mask ^= 1 << trailing_zeros;
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
                push_escaped_str(*ESCAPED.get_unchecked(c), buffer);
                start_ptr = ptr2.add(1);
            }
        }
    }

    if end_ptr > start_ptr {
        let slc = slice::from_raw_parts(start_ptr, end_ptr as usize - start_ptr as usize);
        buffer.push_str(std::str::from_utf8_unchecked(slc));
    }
}
