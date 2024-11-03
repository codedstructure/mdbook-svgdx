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
    Event::{End, Html, Start, Text},
    Tag, TagEnd,
};
use pulldown_cmark_to_cmark::cmark;

use svgdx::TransformConfig;

pub struct SvgdxProc;

impl Preprocessor for SvgdxProc {
    fn name(&self) -> &str {
        "svgdx"
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        // This processor is supported by both html and markdown renderers
        renderer != "not-supported"
    }

    #[rustfmt::skip]
    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book, Error> {
        let mut book = book;

        let mut config = TransformConfig::default();
        if let Some(cfg) = ctx.config.get_preprocessor(self.name()) {
            for (key, value) in cfg.iter() {
                // Messy, but keeps quoted strings quoted and non-string values (e.g. numbers)
                // as strings; goal is we can parse() everything without caring about types.
                let v = if let Some(v) = value.as_str() { v } else { &value.to_string() };

                // Note: not every config key is included here, only those which make sense in an
                // mdbook context - generally those which have a visible effect on the output.
                match key.as_str() {
                    "scale" => { config.scale = v.parse()?; }
                    "border" => { config.border = v.parse()?; }
                    "add-auto-styles" => { config.add_auto_styles = v.parse()?; }
                    "background" => { config.background = v.parse()?; }
                    "seed" => { config.seed = v.parse()?; }
                    "loop-limit" => { config.loop_limit = v.parse()?; }
                    "var-limit" => { config.var_limit = v.parse()?; }
                    "font-size" => { config.font_size = v.parse()?; }
                    "font-family" => { config.font_family = v.parse()?; }
                    "theme" => { config.theme = v.parse()?; }
                    // Command is a valid mdbook preprocessor config key, but we don't use it
                    "command" => {}
                    _ => Err(Error::msg(format!("Unknown config key: {}", key)))?,
                }
            }
        }
        // TODO: add this after next svgdx release
        // config.use_local_styles = true;

        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                if let Ok(processed) = codeblock_parser(chapter, &config) {
                    chapter.content = processed;
                }
            }
        });
        Ok(book)
    }
}

fn codeblock_parser(
    chapter: &mut Chapter,
    config: &TransformConfig,
) -> Result<String, std::fmt::Error> {
    let md_events = mdbook::utils::new_cmark_parser(&chapter.content, false);

    let mut in_block = None;
    let mut events = Vec::new();
    for ev in md_events {
        match (&mut in_block, ev.clone()) {
            (None, Start(Tag::CodeBlock(Fenced(Borrowed(block_type)))))
                if block_type == "svgdx" || block_type.starts_with("svgdx-") =>
            {
                // surround the whole thing in a div with appropriate class so
                // we can style it. Note deliberate empty lines here to get
                // markdown to ignore the fact we've just opened a <div> Html block
                events.push(Html(format!("\n\n<div class='{}'>\n\n", block_type).into()));
                in_block = Some(block_type.to_string());
            }
            (Some(block_type), Text(content)) => {
                if block_type == "svgdx-xml" {
                    // Special case this fence type to display the XML input
                    // prior to the rendered SVG output.
                    events.push(Start(Tag::CodeBlock(CodeBlockKind::Fenced("xml".into()))));
                    events.push(Text(content.clone()));
                    events.push(End(TagEnd::CodeBlock));
                }
                events.push(Start(Tag::Paragraph));
                // Need to avoid blank lines in the rendered SVG, as they can cause
                // markdown to resume 'normal' md processing, especially when e.g.
                // indentation can cause an implicit code block to be started.
                // See https://talk.commonmark.org/t/inline-html-breaks-when-using-indentation/3317
                // and https://spec.commonmark.org/0.31.2/#html-blocks
                let svg_output = svgdx_handler(&content, config)
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .collect::<Vec<_>>()
                    .join("\n");
                events.push(Html(svg_output.into()));
                events.push(End(TagEnd::Paragraph));
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

fn svgdx_handler(s: &str, cfg: &TransformConfig) -> String {
    svgdx::transform_str(s.to_string(), cfg).unwrap_or_else(|e| {
        format!(
            r#"<div style="color: red; border: 5px double red; padding: 1em;">{}</div>"#,
            e.to_string().replace('\n', "<br/>")
        )
    })
}
