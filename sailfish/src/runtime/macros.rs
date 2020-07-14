#[macro_export]
#[doc(hidden)]
macro_rules! render {
    ($ctx:ident, $value:expr) => {
        $crate::runtime::Render::render(&($value), &mut $ctx.buf)?
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! render_escaped {
    ($ctx:ident, $value:expr) => {
        $crate::runtime::Render::render_escaped(&($value), &mut $ctx.buf)?
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! render_text {
    ($ctx:ident, $value:expr) => {
        $ctx.buf.push_str($value)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! render_noop {
    ($ctx:ident, $value:expr) => {};
}
