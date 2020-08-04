use core::slice;

use super::super::utils::memcpy_16;
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

    if likely!(end_ptr > start_ptr) {
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
        if unlikely!(idx < ESCAPED_LEN) {
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

pub(super) unsafe fn escape_small(feed: &str, mut buf: *mut u8) -> usize {
    let mut start_ptr = feed.as_ptr();
    let mut ptr = start_ptr;
    let end_ptr = start_ptr.add(feed.len());
    let buf_begin = buf;

    while ptr < end_ptr {
        debug_assert!(start_ptr <= ptr);
        let idx = *ESCAPE_LUT.get_unchecked(*ptr as usize) as usize;
        debug_assert!(idx <= 9);
        if unlikely!(idx < ESCAPED_LEN) {
            let escaped = ESCAPED.get_unchecked(idx);
            if ptr > start_ptr {
                let len = ptr as usize - start_ptr as usize;

                memcpy_16(start_ptr, buf, len);
                buf = buf.add(len);
            }
            memcpy_16(escaped.as_ptr(), buf, escaped.len());
            buf = buf.add(escaped.len());
            start_ptr = ptr.add(1);
        }
        ptr = ptr.add(1);
    }

    debug_assert_eq!(ptr, end_ptr);
    debug_assert!(start_ptr <= ptr);

    if likely!(end_ptr > start_ptr) {
        let len = end_ptr as usize - start_ptr as usize;
        memcpy_16(start_ptr, buf, len);
        buf = buf.add(len);
    }

    buf as usize - buf_begin as usize
}
