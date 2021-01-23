#![allow(clippy::cast_ptr_alignment)]

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::slice;

use super::super::Buffer;
use super::naive::push_escaped_str;
use super::{ESCAPED, ESCAPED_LEN, ESCAPE_LUT};

const VECTOR_BYTES: usize = std::mem::size_of::<__m256i>();

#[target_feature(enable = "avx2")]
pub unsafe fn escape(feed: &str, buffer: &mut Buffer) {
    debug_assert!(feed.len() >= 16);

    let len = feed.len();
    if len < VECTOR_BYTES {
        escape_small(feed, buffer);
        return;
    }

    let mut start_ptr = feed.as_ptr();
    let mut ptr = start_ptr;
    let end_ptr = feed[len..].as_ptr();

    let v_independent1 = _mm256_set1_epi8(5);
    let v_independent2 = _mm256_set1_epi8(2);
    let v_key1 = _mm256_set1_epi8(0x27);
    let v_key2 = _mm256_set1_epi8(0x3e);

    let maskgen = |x: __m256i| -> u32 {
        _mm256_movemask_epi8(_mm256_or_si256(
            _mm256_cmpeq_epi8(_mm256_or_si256(x, v_independent1), v_key1),
            _mm256_cmpeq_epi8(_mm256_or_si256(x, v_independent2), v_key2),
        )) as u32
    };

    while ptr <= end_ptr.sub(VECTOR_BYTES) {
        let mut mask = maskgen(_mm256_loadu_si256(ptr as *const __m256i));
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

        let mut mask =
            maskgen(_mm256_loadu_si256(ptr.sub(backs) as *const __m256i)) >> backs;
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

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn escape_small(feed: &str, buffer: &mut Buffer) {
    debug_assert!(feed.len() >= 16);
    debug_assert!(feed.len() < VECTOR_BYTES);

    let len = feed.len();
    let mut start_ptr = feed.as_ptr();
    let mut ptr = start_ptr;
    let end_ptr = start_ptr.add(len);

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

    let mut mask = maskgen(_mm_loadu_si128(ptr as *const __m128i));
    while mask != 0 {
        let trailing_zeros = mask.trailing_zeros() as usize;
        mask ^= 1 << trailing_zeros;
        let ptr2 = ptr.add(trailing_zeros);
        let c = ESCAPE_LUT[*ptr2 as usize] as usize;
        if c < ESCAPED_LEN {
            if start_ptr < ptr2 {
                let slc =
                    slice::from_raw_parts(start_ptr, ptr2 as usize - start_ptr as usize);
                buffer.push_str(std::str::from_utf8_unchecked(slc));
            }
            push_escaped_str(*ESCAPED.get_unchecked(c), buffer);
            start_ptr = ptr2.add(1);
        }
    }

    if len != 16 {
        ptr = ptr.add(16);
        let read_ptr = end_ptr.sub(16);
        let backs = 32 - len;
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
