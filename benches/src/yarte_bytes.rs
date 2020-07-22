use bytes::BytesMut;
use std::mem::{ManuallyDrop, MaybeUninit};
use yarte::{Buffer, TemplateBytes};

// Declare CARGO_CFG_HTMLESCAPE_DISABLE_AUTO_SIMD=
#[repr(transparent)]
struct Buff(Vec<u8>);

impl Buff {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    fn cap(&self) -> usize {
        self.0.capacity()
    }
}

impl Buffer for Buff {
    type Freeze = Vec<u8>;

    #[inline]
    fn with_capacity(capacity: usize) -> Self
    where
        Self: Sized,
    {
        Buff(Vec::with_capacity(capacity))
    }

    #[inline]
    fn extend_from_slice(&mut self, src: &[u8]) {
        self.reserve(src.len());
        unsafe {
            debug_assert!(self.cap() - self.len() >= src.len());
            std::ptr::copy_nonoverlapping(src.as_ptr(), self.buf_ptr(), src.len());
            self.advance(src.len())
        }
    }

    #[inline]
    fn reserve(&mut self, additional: usize) {
        debug_assert!(self.len() <= self.cap());
        if std::intrinsics::unlikely(self.cap().wrapping_sub(self.len()) < additional) {
            self.0.reserve(additional);
        }
    }

    #[inline]
    fn freeze(self) -> Self::Freeze {
        self.0
    }

    #[inline]
    unsafe fn advance(&mut self, cnt: usize) {
        self.0.set_len(self.len() + cnt);
    }

    #[inline]
    unsafe fn buf_ptr(&mut self) -> *mut u8 {
        self.0.as_mut_ptr().add(self.len())
    }
}
const SIZE: usize = 109915;
struct Bufff {
    ptr: [MaybeUninit<u8>; SIZE],
    len: usize,
}

impl Bufff {
    #[inline]
    fn len(&self) -> usize {
        self.len
    }

    #[inline]
    fn as_ptr(&mut self) -> *mut u8 {
        self.ptr.as_mut_ptr() as *mut u8
    }
}

impl Buffer for Bufff {
    type Freeze = Vec<u8>;

    #[inline]
    fn with_capacity(capacity: usize) -> Self
    where
        Self: Sized,
    {
        if capacity > SIZE {
            panic!("Max capacity is {}", SIZE);
        }

        Bufff {
            ptr: [MaybeUninit::uninit(); SIZE],
            len: 0,
        }
    }

    #[inline]
    fn extend_from_slice(&mut self, src: &[u8]) {
        self.reserve(src.len());
        unsafe {
            debug_assert!(SIZE - self.len() >= src.len());
            std::ptr::copy_nonoverlapping(src.as_ptr(), self.buf_ptr(), src.len());
            self.advance(src.len())
        }
    }

    #[inline]
    fn reserve(&mut self, additional: usize) {
        debug_assert!(self.len() <= SIZE);
        if std::intrinsics::unlikely(SIZE.wrapping_sub(self.len()) < additional) {
            panic!("Max capacity is {}", SIZE);
        }
    }

    #[inline]
    fn freeze(self) -> Self::Freeze {
        unsafe { Vec::new() }
    }

    #[inline]
    unsafe fn advance(&mut self, cnt: usize) {
        self.len += cnt;
    }

    #[inline]
    unsafe fn buf_ptr(&mut self) -> *mut u8 {
        self.as_ptr().add(self.len())
    }
}

pub fn big_table(b: &mut criterion::Bencher<'_>, size: &usize) {
    let mut table = Vec::with_capacity(*size);
    for _ in 0..*size {
        let mut inner = Vec::with_capacity(*size);
        for i in 0..*size {
            inner.push(i);
        }
        table.push(inner);
    }
    let t = BigTable { table };
    b.iter(|| t.call::<Bufff>(109915));
}

#[derive(TemplateBytes)]
#[template(path = "big-table")]
struct BigTable {
    table: Vec<Vec<usize>>,
}

pub fn teams(b: &mut criterion::Bencher<'_>) {
    let teams = &vec![
        Team {
            name: "Jiangsu".into(),

            score: 43,
        },
        Team {
            name: "Beijing".into(),
            score: 27,
        },
        Team {
            name: "Guangzhou".into(),
            score: 22,
        },
        Team {
            name: "Shandong".into(),
            score: 12,
        },
    ];
    let year = 2015;
    b.iter(|| Teams { teams, year }.ccall::<Buff>(239));
}

#[derive(TemplateBytes)]
#[template(path = "teams")]
struct Teams<'a> {
    year: u16,
    teams: &'a [Team],
}

struct Team {
    name: String,
    score: u8,
}
