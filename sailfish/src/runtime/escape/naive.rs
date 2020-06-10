use core::slice;

use super::super::Buffer;
use super::{ESCAPED, ESCAPED_LEN, ESCAPE_LUT};

#[inline]
pub(super) unsafe fn escape(
    buffer: &mut Buffer,
    mut start_ptr: *const u8,
    ptr: *const u8,
    end_ptr: *const u8,
) {
    start_ptr = proceed(buffer, start_ptr, ptr, end_ptr);

    if end_ptr > start_ptr {
        let slc = slice::from_raw_parts(start_ptr, end_ptr as usize - start_ptr as usize);
        buffer.push_str(std::str::from_utf8_unchecked(slc));
    }
}

#[inline]
pub(super) unsafe fn proceed(
    buffer: &mut Buffer,
    mut start_ptr: *const u8,
    mut ptr: *const u8,
    end_ptr: *const u8,
) -> *const u8 {
    while ptr < end_ptr {
        debug_assert!(start_ptr <= ptr);
        let idx = ESCAPE_LUT[*ptr as usize] as usize;
        debug_assert!(idx <= 9);
        if idx < ESCAPED_LEN {
            if ptr > start_ptr {
                let slc =
                    slice::from_raw_parts(start_ptr, ptr as usize - start_ptr as usize);
                buffer.push_str(std::str::from_utf8_unchecked(slc));
            }
            buffer.push_str(*ESCAPED.get_unchecked(idx));
            start_ptr = ptr.add(1);
        }
        ptr = ptr.add(1);
    }

    debug_assert_eq!(ptr, end_ptr);
    debug_assert!(start_ptr <= ptr);
    start_ptr
}
