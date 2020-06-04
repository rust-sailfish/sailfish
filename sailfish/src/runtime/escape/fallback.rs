use super::naive;

#[cfg(target_pointer_width = "16")]
const USIZE_BYTES: usize = 2;

#[cfg(target_pointer_width = "32")]
const USIZE_BYTES: usize = 4;

#[cfg(target_pointer_width = "64")]
const USIZE_BYTES: usize = 8;

const USIZE_ALIGN: usize = USIZE_BYTES - 1;

#[inline(always)]
fn contains_zero_byte(x: usize) -> bool {
    const LO_U64: u64 = 0x0101010101010101;
    const HI_U64: u64 = 0x8080808080808080;
    const LO_USIZE: usize = LO_U64 as usize;
    const HI_USIZE: usize = HI_U64 as usize;

    x.wrapping_sub(LO_USIZE) & !x & HI_USIZE != 0
}

#[inline]
fn contains_key(x: usize) -> bool {
    const INDEPENDENTS1: usize = 0x0404040404040404_u64 as usize;
    const INDEPENDENTS2: usize = 0x0202020202020202_u64 as usize;
    const KEY1: usize = 0x2626262626262626_u64 as usize;
    const KEY2: usize = 0x3e3e3e3e3e3e3e3e_u64 as usize;

    let y1 = x | INDEPENDENTS1;
    let y2 = x | INDEPENDENTS2;
    let z1 = y1.wrapping_sub(KEY1);
    let z2 = y2.wrapping_sub(KEY2);
    contains_zero_byte(z1) || contains_zero_byte(z2)
}

pub unsafe fn escape<F: FnMut(&str)>(writer: &mut F, bytes: &[u8]) {
    let len = bytes.len();
    let mut start_ptr = bytes.as_ptr();
    let end_ptr = start_ptr.add(len);

    if bytes.len() < USIZE_BYTES {
        naive::escape(writer, start_ptr, start_ptr, end_ptr);
        return;
    }

    let ptr = start_ptr;
    let aligned_ptr = ptr.add(USIZE_BYTES - (start_ptr as usize & USIZE_ALIGN));
    debug_assert_eq!(aligned_ptr as usize % USIZE_BYTES, 0);
    debug_assert!(aligned_ptr <= end_ptr);

    let chunk = (ptr as *const usize).read_unaligned();
    if contains_key(chunk) {
        start_ptr = naive::proceed(writer, start_ptr, ptr, aligned_ptr);
    }

    escape_aligned(writer, start_ptr, aligned_ptr, end_ptr);
}

pub unsafe fn escape_aligned<F: FnMut(&str)>(
    writer: &mut F,
    mut start_ptr: *const u8,
    mut ptr: *const u8,
    end_ptr: *const u8,
) {
    while ptr.add(USIZE_BYTES) <= end_ptr {
        debug_assert_eq!((ptr as usize) % USIZE_BYTES, 0);

        let chunk = *(ptr as *const usize);
        if contains_key(chunk) {
            start_ptr = naive::proceed(writer, start_ptr, ptr, ptr.add(USIZE_BYTES))
        }
        ptr = ptr.add(USIZE_BYTES);
    }
    debug_assert!(ptr <= end_ptr);
    debug_assert!(start_ptr <= ptr);
    naive::escape(writer, start_ptr, ptr, end_ptr);
}
