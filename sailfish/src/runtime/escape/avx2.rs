#![allow(clippy::cast_ptr_alignment)]

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::slice;

use super::super::Buffer;
use super::{naive, sse2};
use super::{ESCAPED, ESCAPED_LEN, ESCAPE_LUT};

const VECTOR_BYTES: usize = std::mem::size_of::<__m256i>();
const VECTOR_ALIGN: usize = VECTOR_BYTES - 1;

#[target_feature(enable = "avx2")]
pub unsafe fn escape(buffer: &mut Buffer, bytes: &[u8]) {
    let len = bytes.len();

    if len < 8 {
        let start_ptr = bytes.as_ptr();
        let end_ptr = start_ptr.add(len);
        naive::escape(buffer, start_ptr, start_ptr, end_ptr);
        return;
    } else if len < VECTOR_BYTES {
        sse2::escape(buffer, bytes);
        return;
    }

    let mut start_ptr = bytes.as_ptr();
    let end_ptr = start_ptr.add(len);

    let v_independent1 = _mm256_set1_epi8(4);
    let v_independent2 = _mm256_set1_epi8(2);
    let v_key1 = _mm256_set1_epi8(0x26);
    let v_key2 = _mm256_set1_epi8(0x3e);

    let maskgen = |x: __m256i| -> i32 {
        _mm256_movemask_epi8(_mm256_or_si256(
            _mm256_cmpeq_epi8(_mm256_or_si256(x, v_independent1), v_key1),
            _mm256_cmpeq_epi8(_mm256_or_si256(x, v_independent2), v_key2),
        ))
    };

    let mut ptr = start_ptr;
    let aligned_ptr = ptr.add(VECTOR_BYTES - (start_ptr as usize & VECTOR_ALIGN));

    {
        let mut mask = maskgen(_mm256_loadu_si256(ptr as *const __m256i));
        loop {
            let trailing_zeros = mask.trailing_zeros() as usize;
            let ptr2 = ptr.add(trailing_zeros);
            if ptr2 >= aligned_ptr {
                break;
            }

            let c = ESCAPE_LUT[*ptr2 as usize] as usize;
            debug_assert!(c < ESCAPED_LEN);
            if start_ptr < ptr2 {
                let slc =
                    slice::from_raw_parts(start_ptr, ptr2 as usize - start_ptr as usize);
                buffer.push_str(std::str::from_utf8_unchecked(slc));
            }
            buffer.push_str(*ESCAPED.get_unchecked(c));
            start_ptr = ptr2.add(1);
            mask ^= 1 << trailing_zeros;
        }
    }

    ptr = aligned_ptr;
    let mut next_ptr = ptr.add(VECTOR_BYTES);

    while next_ptr <= end_ptr {
        let mut mask = maskgen(_mm256_load_si256(ptr as *const __m256i));
        while mask != 0 {
            let trailing_zeros = mask.trailing_zeros() as usize;
            let ptr2 = ptr.add(trailing_zeros);
            let c = ESCAPE_LUT[*ptr2 as usize] as usize;
            debug_assert!(c < ESCAPED_LEN);
            if start_ptr < ptr2 {
                let slc =
                    slice::from_raw_parts(start_ptr, ptr2 as usize - start_ptr as usize);
                buffer.push_str(std::str::from_utf8_unchecked(slc));
            }
            buffer.push_str(*ESCAPED.get_unchecked(c));
            start_ptr = ptr2.add(1);
            mask ^= 1 << trailing_zeros;
        }

        ptr = next_ptr;
        next_ptr = next_ptr.add(VECTOR_BYTES);
    }

    sse2::escape_aligned(buffer, start_ptr, ptr, end_ptr);
}
