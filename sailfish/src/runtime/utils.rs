use std::ptr;

macro_rules! cfg_json {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "json")]
            #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
            $item
        )*
    }
}

#[cfg(sailfish_nightly)]
macro_rules! likely {
    ($val:expr) => {
        std::intrinsics::likely($val)
    };
}

#[cfg(not(sailfish_nightly))]
macro_rules! likely {
    ($val:expr) => {
        $val
    };
}

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

/// Custom memcpy implementation is faster on some platforms
/// implementation based on glibc (https://github.molgen.mpg.de/git-mirror/glibc/blob/master/sysdeps/x86_64/multiarch/memcpy-avx-unaligned.S)
#[allow(clippy::cast_ptr_alignment)]
#[cfg(any(target_arch = "x86_64", target_arch = "x86", target_arch = "aarch64"))]
pub unsafe fn memcpy_16(src: *const u8, dst: *mut u8, len: usize) {
    debug_assert!(len <= 16);
    let len_u8 = len as u8;

    if len_u8 >= 8 {
        let offset = len - 8;
        let t2 = ptr::read_unaligned(src.add(offset) as *const u64);
        let t1 = ptr::read_unaligned(src as *const u64);
        ptr::write_unaligned(dst.add(offset) as *mut u64, t2);
        ptr::write_unaligned(dst as *mut u64, t1);
    } else if len_u8 >= 4 {
        let offset = len - 4;
        let t2 = ptr::read_unaligned(src.add(offset) as *const u32);
        let t1 = ptr::read_unaligned(src as *const u32);
        ptr::write_unaligned(dst.add(offset) as *mut u32, t2);
        ptr::write_unaligned(dst as *mut u32, t1);
    } else if len_u8 >= 2 {
        let offset = len - 2;
        let t2 = ptr::read_unaligned(src.add(offset) as *const u16);
        let t1 = ptr::read_unaligned(src as *const u16);
        ptr::write_unaligned(dst.add(offset) as *mut u16, t2);
        ptr::write_unaligned(dst as *mut u16, t1);
    } else if len_u8 >= 1 {
        *dst = *src;
    }
}

#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "aarch64"
)))]
pub use ptr::copy_nonoverlapping as memcpy_16;
