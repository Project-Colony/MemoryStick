use comrak::{markdown_to_html, ExtensionOptionsBuilder, Options, ParseOptionsBuilder, RenderOptionsBuilder};

pub fn render(markdown: &str) -> String {
    let extension = ExtensionOptionsBuilder::default()
        .strikethrough(true)
        .table(true)
        .autolink(true)
        .tasklist(true)
        .superscript(false) // off: clashes with ^ in LaTeX/KaTeX
        .footnotes(true)
        .description_lists(true)
        .math_dollars(true) // $...$ and $$...$$ become data-math-style spans
        .math_code(true)    // ```math becomes a math block
        .header_ids(Some("h-".to_string()))
        .front_matter_delimiter(Some("---".to_string()))
        .build()
        .unwrap_or_default();

    let parse = ParseOptionsBuilder::default()
        .smart(true)
        .build()
        .unwrap_or_default();

    let render = RenderOptionsBuilder::default()
        .unsafe_(true)
        .hardbreaks(false)
        .build()
        .unwrap_or_default();

    let opts = Options {
        extension,
        parse,
        render,
    };

    markdown_to_html(markdown, &opts)
}
