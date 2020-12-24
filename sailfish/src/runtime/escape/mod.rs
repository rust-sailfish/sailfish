//! HTML escaping utilities
//!
//! By default sailfish replaces the characters `&"'<>` with the equivalent html.

#![cfg_attr(
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        not(miri),
        target_feature = "avx2"
    ),
    allow(dead_code)
)]

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(miri)))]
mod avx2;
mod fallback;
mod naive;
#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(miri)))]
mod sse2;

static ESCAPE_LUT: [u8; 256] = [
    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9, 9, 9, 0, 9, 9, 9, 1, 2, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9, 3, 9, 4, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9,
];

const ESCAPED: [&str; 5] = ["&quot;", "&amp;", "&#039;", "&lt;", "&gt;"];
const ESCAPED_LEN: usize = 5;

use super::buffer::Buffer;

/// write the escaped contents into `Buffer`
#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(miri)))]
#[cfg_attr(feature = "perf-inline", inline)]
pub fn escape_to_buf(feed: &str, buf: &mut Buffer) {
    #[cfg(not(target_feature = "avx2"))]
    {
        use std::sync::atomic::{AtomicPtr, Ordering};

        type FnRaw = *mut ();
        static FN: AtomicPtr<()> = AtomicPtr::new(detect as FnRaw);

        fn detect(feed: &str, buf: &mut Buffer) {
            debug_assert!(feed.len() >= 16);
            let fun = if is_x86_feature_detected!("avx2") {
                avx2::escape
            } else if is_x86_feature_detected!("sse2") {
                sse2::escape
            } else {
                fallback::escape
            };

            FN.store(fun as FnRaw, Ordering::Relaxed);
            unsafe { fun(feed, buf) };
        }

        unsafe {
            if feed.len() < 16 {
                buf.reserve_small(feed.len() * 6);
                let l = naive::escape_small(feed, buf.as_mut_ptr().add(buf.len()));
                buf.advance(l);
            } else {
                let fun = FN.load(Ordering::Relaxed);
                std::mem::transmute::<FnRaw, fn(&str, &mut Buffer)>(fun)(feed, buf);
            }
        }
    }

    #[cfg(target_feature = "avx2")]
    unsafe {
        if feed.len() < 16 {
            buf.reserve_small(feed.len() * 6);
            let l = naive::escape_small(feed, buf.as_mut_ptr().add(buf.len()));
            buf.advance(l);
        } else if cfg!(target_feature = "avx2") {
            avx2::escape(feed, buf);
        }
    }
}

/// write the escaped contents into `Buffer`
#[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), not(miri))))]
#[cfg_attr(feature = "perf-inline", inline)]
pub fn escape_to_buf(feed: &str, buf: &mut Buffer) {
    unsafe {
        if cfg!(miri) {
            let bp = feed.as_ptr();
            naive::escape(buf, bp, bp, bp.add(feed.len()))
        } else if feed.len() < 16 {
            buf.reserve_small(feed.len() * 6);
            let l = naive::escape_small(feed, buf.as_mut_ptr().add(buf.len()));
            buf.advance(l);
        } else {
            fallback::escape(feed, buf)
        }
    }
}

/// write the escaped contents into `String`
///
/// # Examples
///
/// ```
/// use sailfish::runtime::escape::escape_to_string;
///
/// let mut buf = String::new();
/// escape_to_string("<h1>Hello, world!</h1>", &mut buf);
/// assert_eq!(buf, "&lt;h1&gt;Hello, world!&lt;/h1&gt;");
/// ```
#[inline]
pub fn escape_to_string(feed: &str, s: &mut String) {
    let mut s2 = String::new();
    std::mem::swap(s, &mut s2);
    let mut buf = Buffer::from(s2);
    escape_to_buf(feed, &mut buf);
    let mut s2 = buf.into_string();
    std::mem::swap(s, &mut s2);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn escape(feed: &str) -> String {
        let mut s = String::new();
        escape_to_string(feed, &mut s);
        s
    }

    #[test]
    fn noescape() {
        assert_eq!(escape(""), "");
        assert_eq!(escape("1234567890"), "1234567890");
        assert_eq!(
            escape("abcdefghijklmnopqrstrvwxyz"),
            "abcdefghijklmnopqrstrvwxyz"
        );
        assert_eq!(escape("!#$%()*+,-.:;=?_^"), "!#$%()*+,-.:;=?_^");
        assert_eq!(
            escape("漢字はエスケープしないはずだよ"),
            "漢字はエスケープしないはずだよ"
        );
    }

    #[test]
    fn escape_short() {
        assert_eq!(escape("<"), "&lt;");
        assert_eq!(escape("\"&<>'"), "&quot;&amp;&lt;&gt;&#039;");
        assert_eq!(
            escape("{\"title\": \"This is a JSON!\"}"),
            "{&quot;title&quot;: &quot;This is a JSON!&quot;}"
        );
        assert_eq!(
            escape("<html><body><h1>Hello, world</h1></body></html>"),
            "&lt;html&gt;&lt;body&gt;&lt;h1&gt;Hello, world&lt;/h1&gt;\
            &lt;/body&gt;&lt;/html&gt;"
        );
    }

    #[test]
    #[rustfmt::skip]
    fn escape_long() {
        assert_eq!(
            escape(r###"m{jml&,?6>\2~08g)\=3`,_`$1@0{i5j}.}2ki\^t}k"'@p4$~?;!;pn_l8v."ki`%/&^=\[y+qcerr`@3*|?du.\0vd#40'.>bcpf\u@m|c<2t7`hk)^?"0u{v%9}4y2hhv?%-f`<;rzwx`7}l(j2b:c\<|z&$x{+k;f`0+w3e0\m.wmdli>94e2hp\$}j0&m(*h$/lwlj#}99r';o.kj@1#}~v+;y~b[~m.eci}&l7fxt`\\{~#k*9z/d{}(.^j}[(,]:<\h]9k2+0*w60/|23~5;/!-h&ci*~e1h~+:1lhh\>y_*>:-\zzv+8uo],,a^k3_,uip]-/.-~\t51a*<{6!<(_|<#o6=\h1*`[2x_?#-/])x};};r@wqx|;/w&jrv~?\`t:^/dug3(g(ener?!t$}h4:57ptnm@71e=t>@o*"$]799r=+)t>co?rvgk%u0c@.9os;#t_*/gqv<za&~r^]"{t4by2t`<q4bfo^&!so5/~(nxk:7l\;#0w41u~w3i$g|>e/t;o<*`~?3.jyx+h)+^cn^j4td|>)~rs)vm#]:"&\fi;54%+z~fhe|w~\q|ui={54[b9tg*?@]g+q!mq]3jg2?eoo"chyat3k#7pq1u=.l]c14twa4tg#5k_""###),
            r###"m{jml&amp;,?6&gt;\2~08g)\=3`,_`$1@0{i5j}.}2ki\^t}k&quot;&#039;@p4$~?;!;pn_l8v.&quot;ki`%/&amp;^=\[y+qcerr`@3*|?du.\0vd#40&#039;.&gt;bcpf\u@m|c&lt;2t7`hk)^?&quot;0u{v%9}4y2hhv?%-f`&lt;;rzwx`7}l(j2b:c\&lt;|z&amp;$x{+k;f`0+w3e0\m.wmdli&gt;94e2hp\$}j0&amp;m(*h$/lwlj#}99r&#039;;o.kj@1#}~v+;y~b[~m.eci}&amp;l7fxt`\\{~#k*9z/d{}(.^j}[(,]:&lt;\h]9k2+0*w60/|23~5;/!-h&amp;ci*~e1h~+:1lhh\&gt;y_*&gt;:-\zzv+8uo],,a^k3_,uip]-/.-~\t51a*&lt;{6!&lt;(_|&lt;#o6=\h1*`[2x_?#-/])x};};r@wqx|;/w&amp;jrv~?\`t:^/dug3(g(ener?!t$}h4:57ptnm@71e=t&gt;@o*&quot;$]799r=+)t&gt;co?rvgk%u0c@.9os;#t_*/gqv&lt;za&amp;~r^]&quot;{t4by2t`&lt;q4bfo^&amp;!so5/~(nxk:7l\;#0w41u~w3i$g|&gt;e/t;o&lt;*`~?3.jyx+h)+^cn^j4td|&gt;)~rs)vm#]:&quot;&amp;\fi;54%+z~fhe|w~\q|ui={54[b9tg*?@]g+q!mq]3jg2?eoo&quot;chyat3k#7pq1u=.l]c14twa4tg#5k_&quot;"###
        );
    }

    #[test]
    #[cfg(not(miri))]
    fn random() {
        const ASCII_CHARS: &'static [u8] = br##"abcdefghijklmnopqrstuvwxyz0123456789-^\@[;:],./\!"#$%&'()~=~|`{+*}<>?_"##;
        let mut state = 88172645463325252u64;
        let mut data = Vec::with_capacity(100);

        let mut buf_naive = Buffer::new();
        let mut buf = Buffer::new();

        for len in 16..100 {
            for _ in 0..5 {
                data.clear();
                for _ in 0..len {
                    // xorshift
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;

                    let idx = state as usize % ASCII_CHARS.len();
                    data.push(ASCII_CHARS[idx]);
                }

                let s = unsafe { std::str::from_utf8_unchecked(&*data) };

                unsafe {
                    naive::escape(
                        &mut buf_naive,
                        s.as_ptr(),
                        s.as_ptr(),
                        s.as_ptr().add(s.len()),
                    );

                    fallback::escape(s, &mut buf);
                    assert_eq!(buf.as_str(), buf_naive.as_str());
                    buf.clear();

                    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                    {
                        if is_x86_feature_detected!("sse2") {
                            sse2::escape(s, &mut buf);
                            assert_eq!(buf.as_str(), buf_naive.as_str());
                            buf.clear();
                        }

                        if is_x86_feature_detected!("avx2") {
                            avx2::escape(s, &mut buf);
                            assert_eq!(buf.as_str(), buf_naive.as_str());
                            buf.clear();
                        }
                    }
                }

                buf_naive.clear();
            }
        }
    }
}
