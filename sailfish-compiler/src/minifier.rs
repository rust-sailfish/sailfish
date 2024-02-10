use minify_html::Cfg;

#[derive(Clone)]
pub struct Minifier {
    // Whether inline css should be minified (<style>..</style>)
    minify_css: bool,
    // Whether inline javascript should be minified (<script>..</script>)
    minify_js: bool,
}

impl Minifier {
    pub fn new() -> Self {
        Self {
            minify_css: false,
            minify_js: false,
        }
    }

    #[inline]
    pub fn minify_css(mut self, new: bool) -> Self {
        self.minify_css = new;
        self
    }

    #[inline]
    pub fn minify_js(mut self, new: bool) -> Self {
        self.minify_js = new;
        self
    }

    #[inline]
    pub fn minify(&self, input: &str) -> String {
        let output = minify_html::minify(
            input.as_bytes(),
            &Cfg {
                do_not_minify_doctype: true,
                preserve_chevron_percent_template_syntax: true,
                minify_css: self.minify_css,
                minify_js: self.minify_js,
                ..Default::default()
            },
        );

        String::from_utf8(output).unwrap()
    }
}
