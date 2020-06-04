use std::fmt;

#[derive(PartialEq, Eq)]
pub struct PrettyString<'a>(pub &'a str);

/// Make diff to display string as multi-line string
impl<'a> fmt::Debug for PrettyString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0)
    }
}

#[macro_export]
macro_rules! assert_string_eq {
    ($left:expr, $right:expr) => {
        pretty_assertions::assert_eq!(
            $crate::PrettyString($left),
            $crate::PrettyString($right)
        );
    };
}
