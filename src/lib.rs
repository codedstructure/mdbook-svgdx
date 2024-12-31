//! An [mdbook](https://rust-lang.github.io/mdBook/) preprocessor for
//! [svgdx](https://github.com/codedstructure/svgdx) fenced code blocks.
//!
//! Example markdown:
//!
//! ~~~markdown
//! # svgdx code block example
//!
//! ```svgdx
//! <svg>
//!  <rect wh="20 5" text="Hello World!"/>
//! </svg>
//! ```
//!
//! The above code block will be transformed into an inline SVG image.
//! ~~~
//!
//! For more information on mdbook preprocessors, including a nop-processor which
//! this is heavily based on, see the
//! [preprocessor developer docs](https://rust-lang.github.io/mdBook/for_developers/preprocessors.html)

use mdbook::book::{Book, Chapter};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::BookItem;

use pulldown_cmark::CodeBlockKind;
use pulldown_cmark::{
    CodeBlockKind::Fenced,
    CowStr::Borrowed,
    Event,
    Event::{End, Html, Start, Text},
    Tag, TagEnd,
};
use pulldown_cmark_to_cmark::cmark;

pub struct SvgdxProc;

impl Preprocessor for SvgdxProc {
    fn name(&self) -> &str {
        "svgdx"
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        // This processor is supported by both html and markdown renderers
        renderer != "not-supported"
    }

    fn run(&self, _: &PreprocessorContext, book: Book) -> Result<Book, Error> {
        let mut book = book;
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                if let Ok(processed) = codeblock_parser(chapter) {
                    chapter.content = processed;
                }
            }
        });
        Ok(book)
    }
}

fn inject_xml(events: &mut Vec<Event>, content: &str) {
    events.push(Html("\n\n<div>\n".into()));
    events.push(Start(Tag::CodeBlock(CodeBlockKind::Fenced("xml".into()))));
    events.push(Text(content.to_owned().into()));
    events.push(End(TagEnd::CodeBlock));
    events.push(Html("\n</div>\n".into()));
}

fn inject_svgdx(events: &mut Vec<Event>, content: &str) {
    events.push(Start(Tag::Paragraph));
    // Need to avoid blank lines in the rendered SVG, as they can cause
    // markdown to resume 'normal' md processing, especially when e.g.
    // indentation can cause an implicit code block to be started.
    // See https://talk.commonmark.org/t/inline-html-breaks-when-using-indentation/3317
    // and https://spec.commonmark.org/0.31.2/#html-blocks
    let svg_output = svgdx_handler(content)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    events.push(Html(svg_output.into()));
    events.push(End(TagEnd::Paragraph));
}

fn codeblock_parser(chapter: &mut Chapter) -> Result<String, std::fmt::Error> {
    let md_events = mdbook::utils::new_cmark_parser(&chapter.content, false);

    let mut in_block = None;
    let mut events = Vec::new();
    for ev in md_events {
        match (&mut in_block, ev.clone()) {
            (None, Start(Tag::CodeBlock(Fenced(Borrowed(block_type)))))
                if matches!(
                    block_type,
                    "svgdx" | "svgdx-xml" | "xml-svgdx" | "svgdx-xml-inline" | "xml-svgdx-inline"
                ) =>
            {
                // surround the whole thing in a div with appropriate class so
                // we can style it. Note deliberate empty lines here to get
                // markdown to ignore the fact we've just opened a <div> Html block
                let style = if block_type.ends_with("-inline") {
                    "style='display: flex; justify-content: space-around;' "
                } else {
                    ""
                };
                events.push(Html(
                    format!("\n\n<div {}class='{}'>\n", style, block_type).into(),
                ));
                in_block = Some(block_type.to_string());
            }
            (Some(block_type), Text(content)) => {
                if block_type.starts_with("xml-svgdx") {
                    // Special case this fence type to display the XML input
                    // prior to the rendered SVG output.
                    inject_xml(&mut events, &content);
                }
                inject_svgdx(&mut events, &content);
                if block_type.starts_with("svgdx-xml") {
                    // Special case this fence type to display the XML input
                    // prior to the rendered SVG output.
                    inject_xml(&mut events, &content);
                }
            }
            (Some(_), End(TagEnd::CodeBlock)) => {
                events.push(Html("</div>".into()));
                in_block = None;
            }
            _ => events.push(ev),
        }
    }
    let mut buf = String::new();
    cmark(&mut events.iter(), &mut buf)?;
    Ok(buf)
}

fn svgdx_handler(s: &str) -> String {
    let cfg = svgdx::TransformConfig {
        svg_style: Some("max-width: 100%; height: auto;".to_string()),
        use_local_styles: true,
        scale: 1.5,
        ..Default::default()
    };
    svgdx::transform_str(s.to_string(), &cfg).unwrap_or_else(|e| {
        format!(
            r#"<div style="color: red; border: 5px double red; padding: 1em;">{}</div>"#,
            e.to_string().replace('\n', "<br/>")
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use assertables::assert_contains;

    #[test]
    fn process_basic_svgdx() {
        let content = r##"
Some **markdown** text

```svgdx
<svg>
  <rect wh="20 5"/>
</svg>
```
"##;

        let expected1 = r##"Some **markdown** text

<div class='svgdx'>


<svg "##;
        let expected2 = r##"
  <rect width="20" height="5"/>
</svg></div>"##;
        let mut chapter = Chapter::new("test", content.to_owned(), ".", Vec::new());
        let result = codeblock_parser(&mut chapter).unwrap();
        assert_contains!(result, expected1);
        assert_contains!(result, expected2);

        let mut z = Book::new();
        z.push_item(chapter);
    }
}
