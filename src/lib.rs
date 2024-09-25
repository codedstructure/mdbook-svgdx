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

use pulldown_cmark::{
    CodeBlockKind::Fenced,
    CowStr::Borrowed,
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

fn codeblock_parser(chapter: &mut Chapter) -> Result<String, std::fmt::Error> {
    let md_events = mdbook::utils::new_cmark_parser(&chapter.content, false);

    let mut in_block = false;
    let mut events = Vec::new();
    for ev in md_events {
        match (&mut in_block, &ev) {
            (false, Start(Tag::CodeBlock(Fenced(Borrowed("svgdx"))))) => {
                events.push(Start(Tag::Paragraph));
                in_block = true;
            }
            (true, Text(content)) => events.push(Html(svgdx_handler(content).into())),
            (true, End(TagEnd::CodeBlock)) => {
                in_block = false;
                events.push(End(TagEnd::Paragraph))
            }
            _ => events.push(ev),
        }
    }
    let mut buf = String::new();
    cmark(&mut events.iter(), &mut buf)?;
    Ok(buf)
}

fn svgdx_handler(s: &str) -> String {
    svgdx::transform_string(s.to_string()).unwrap_or_else(|e| {
        format!(
            r#"<div style="color: red; border: 5px double red; padding: 1em;">{}</div>"#,
            e.replace('\n', "<br/>")
        )
    })
}
