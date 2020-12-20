#[macro_export]
#[doc(hidden)]
macro_rules! render {
    ($buf:ident, $value:expr) => {
        $crate::runtime::Render::render(&($value), $buf)?
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! render_escaped {
    ($buf:ident, $value:expr) => {
        $crate::runtime::Render::render_escaped(&($value), $buf)?
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! render_text {
    ($buf:ident, $value:expr) => {
        $buf.push_str($value)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! render_noop {
    ($buf:ident, $value:expr) => {};
}
