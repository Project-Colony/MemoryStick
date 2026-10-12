use comrak::{markdown_to_html, Options};

pub fn render(markdown: &str) -> String {
    let mut opts = Options::default();
    let ext = &mut opts.extension;
    ext.strikethrough = true;
    ext.table = true;
    ext.autolink = true;
    ext.tasklist = true;
    // superscript stays off: it clashes with ^ in LaTeX/KaTeX
    ext.footnotes = true;
    ext.description_lists = true;
    ext.math_dollars = true; // $...$ and $$...$$ become data-math-style spans
    ext.math_code = true; // ```math becomes a math block
    ext.header_id_prefix = Some("h-".to_string());
    ext.front_matter_delimiter = Some("---".to_string());
    opts.parse.smart = true;
    // Raw HTML is kept as written (kbd, details, sub...). render() in app.js
    // drops the elements that could act on the app's own page, such as
    // scripts, frames and style sheets, before the document is shown.
    opts.render.r#unsafe = true;

    markdown_to_html(markdown, &opts)
}

#[cfg(test)]
mod tests {
    use super::render;

    // What app.js relies on: heading ids for the table of contents and
    // in-page links, the data-math-style markers KaTeX looks for, and front
    // matter left out of the page.
    #[test]
    fn renders_what_the_interface_expects() {
        let html = render("---\ntitle: x\n---\n# Intro\n\n$a^2$\n\n```math\nb\n```\n");
        assert!(!html.contains("title: x"), "{html}");
        assert!(html.contains("<h1 id=\"h-intro\">"), "{html}");
        assert!(html.contains("data-math-style=\"inline\""), "{html}");
        assert!(html.contains("data-math-style=\"display\""), "{html}");
    }
}
