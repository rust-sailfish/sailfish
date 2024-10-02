module.exports = grammar({
  name: "sailfish",

  rules: {
    document: $ => repeat($._node),

    _node: $ => choice(
      $.html_part,
      $.sailfish_part,
      $.comment,
    ),

    html_part: $ => /([^<]|<[^%])+/,

    sailfish_part: $ => seq(/<%[-=+ ]/, $.rust_code, "%>"),

    rust_code: $ => /([^%]|%[^>])+/,
    
    comment: $ => /<%#.*%>/,
  },
});
