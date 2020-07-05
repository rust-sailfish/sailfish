use core::ptr;
use core::slice;

use super::super::Buffer;
use super::{ESCAPED, ESCAPED_LEN, ESCAPE_LUT};

#[cfg(sailfish_nightly)]
macro_rules! unlikely {
    ($val:expr) => {
        std::intrinsics::unlikely($val)
    };
}

#[cfg(not(sailfish_nightly))]
macro_rules! unlikely {
    ($val:expr) => {
        $val
    };
}

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
                let slc =
                    slice::from_raw_parts(start_ptr, ptr as usize - start_ptr as usize);

                memcpy_small(slc.as_ptr(), buf, slc.len());
                buf = buf.add(slc.len());
            }
            memcpy_small(escaped.as_ptr(), buf, escaped.len());
            buf = buf.add(escaped.len());
            start_ptr = ptr.add(1);
        }
        ptr = ptr.add(1);
    }

    debug_assert_eq!(ptr, end_ptr);
    debug_assert!(start_ptr <= ptr);

    if end_ptr > start_ptr {
        let slc = slice::from_raw_parts(start_ptr, end_ptr as usize - start_ptr as usize);
        memcpy_small(slc.as_ptr(), buf, slc.len());
        buf = buf.add(slc.len());
    }

    buf as usize - buf_begin as usize
}

/// memcpy implementation based on glibc (https://github.molgen.mpg.de/git-mirror/glibc/blob/master/sysdeps/x86_64/multiarch/memcpy-avx-unaligned.S)
#[cfg_attr(feature = "perf-inline", inline)]
#[allow(clippy::cast_ptr_alignment)]
unsafe fn memcpy_small(src: *const u8, dst: *mut u8, len: usize) {
    debug_assert!(len <= 16);
    if len >= 8 {
        let tmp = ptr::read_unaligned(src as *const u64);
        ptr::write_unaligned(dst as *mut u64, tmp);
        let offset = len - 8;
        let tmp = ptr::read_unaligned(src.add(offset) as *const u64);
        ptr::write_unaligned(dst.add(offset) as *mut u64, tmp);
    } else if len >= 4 {
        let tmp = ptr::read_unaligned(src as *const u32);
        ptr::write_unaligned(dst as *mut u32, tmp);
        let offset = len - 4;
        let tmp = ptr::read_unaligned(src.add(offset) as *const u32);
        ptr::write_unaligned(dst.add(offset) as *mut u32, tmp);
    } else if len >= 2 {
        let tmp = ptr::read_unaligned(src as *const u16);
        ptr::write_unaligned(dst as *mut u16, tmp);
        let offset = len - 2;
        let tmp = ptr::read_unaligned(src.add(offset) as *const u16);
        ptr::write_unaligned(dst.add(offset) as *mut u16, tmp);
    } else if len >= 1 {
        *dst = *src;
    }
}
