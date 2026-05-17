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

use mdbook_markdown::MarkdownOptions;
use mdbook_preprocessor::book::BookItem;
use mdbook_preprocessor::book::{Book, Chapter};
use mdbook_preprocessor::errors::{Error, Result};
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};

use pulldown_cmark::CodeBlockKind;
use pulldown_cmark::{
    CodeBlockKind::Fenced,
    CowStr::Borrowed,
    Event,
    Event::{End, Html, Start, Text},
    Tag, TagEnd,
};
use pulldown_cmark_to_cmark::cmark;
use std::ffi::OsString;
use std::io::Write;
use std::process::{Command, Stdio};

#[cfg(feature = "svgdx-builtin")]
use svgdx::AutoStyleMode;

pub struct SvgdxProc;

pub const SVGDX_BIN_ENV_VAR: &str = "MDBOOK_SVGDX_BIN";
const SVGDX_AUTO_STYLE_MODE: &str = "inline";
const SVGDX_SCALE: &str = "1.5";
const SVGDX_STYLE: &str = "min-width: 25%; max-width: 100%; height: auto;";

#[derive(Debug, PartialEq, Eq)]
enum SvgdxBackend {
    External(OsString),
    #[cfg(feature = "svgdx-builtin")]
    Builtin,
}

impl SvgdxBackend {
    fn from_env() -> Result<Self> {
        Self::resolve(|| std::env::var_os(SVGDX_BIN_ENV_VAR))
    }

    fn resolve(get_env: impl FnOnce() -> Option<OsString>) -> Result<Self> {
        match get_env() {
            Some(program) if !program.is_empty() => return Ok(Self::External(program)),
            Some(_) => {
                return Err(Error::msg(format!(
                    "Environment variable {SVGDX_BIN_ENV_VAR} is set but empty; it should point to an svgdx executable"
                )));
            }
            None => {}
        }

        #[cfg(feature = "svgdx-builtin")]
        {
            Ok(Self::Builtin)
        }

        #[cfg(not(feature = "svgdx-builtin"))]
        {
            Err(Error::msg(format!(
                "No svgdx backend is available. Set {SVGDX_BIN_ENV_VAR} to the svgdx executable path, or rebuild mdbook-svgdx with the `svgdx-builtin` feature enabled"
            )))
        }
    }

    fn render(&self, input: &str) -> Result<String> {
        match self {
            Self::External(program) => render_external_svgdx(program, input),
            #[cfg(feature = "svgdx-builtin")]
            Self::Builtin => Ok(render_builtin_svgdx(input)),
        }
    }
}

impl Preprocessor for SvgdxProc {
    fn name(&self) -> &str {
        "svgdx"
    }

    fn supports_renderer(&self, renderer: &str) -> Result<bool> {
        // This processor is supported by both html and markdown renderers
        Ok(renderer != "not-supported")
    }

    fn run(&self, _: &PreprocessorContext, book: Book) -> Result<Book, Error> {
        let backend = SvgdxBackend::from_env()?;
        let mut book = book;
        let mut parse_error = None;
        book.for_each_mut(|item| {
            if parse_error.is_some() {
                return;
            }
            if let BookItem::Chapter(chapter) = item {
                match codeblock_parser(chapter, &backend) {
                    Ok(processed) => chapter.content = processed,
                    Err(err) => parse_error = Some(err),
                }
            }
        });
        if let Some(err) = parse_error {
            return Err(err);
        }
        Ok(book)
    }
}

fn codeblock_parser(chapter: &Chapter, backend: &SvgdxBackend) -> Result<String> {
    let md_events =
        mdbook_markdown::new_cmark_parser(&chapter.content, &MarkdownOptions::default());

    let mut in_block = None;
    let mut events = Vec::new();
    let mut block_content = Vec::new();
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
                let style = "style='display: flex; flex-wrap: wrap; justify-content: space-around; align-items: center;' ";
                events.push(Html(
                    format!("\n\n<div {style}class='{block_type}'>\n").into(),
                ));
                in_block = Some(block_type.to_string());
            }
            (Some(_), Text(content)) => {
                // content of code block isn't necessarily in a single Text event;
                // CRLF sources in particular seem to generate a Text event per line.
                block_content.push(content.clone());
            }
            (Some(block_type), End(TagEnd::CodeBlock)) => {
                handle_content(&block_content.concat(), block_type, &mut events, backend)?;
                events.push(Html("</div>".into()));
                block_content.clear();
                in_block = None;
            }
            _ => events.push(ev),
        }
    }
    if let Some(block_type) = in_block {
        // The CommonMark spec allows for non-terminated code blocks, treating
        // the end of the document as an implicit end-of-fence.
        // https://spec.commonmark.org/0.31.2/#fenced-code-blocks
        handle_content(&block_content.concat(), &block_type, &mut events, backend)?;
        events.push(Html("</div>".into()));
    }

    let mut buf = String::new();
    cmark(&mut events.iter(), &mut buf)?;
    Ok(buf)
}

fn handle_content(
    content: &str,
    block_type: &str,
    events: &mut Vec<Event>,
    backend: &SvgdxBackend,
) -> Result<()> {
    if block_type.starts_with("xml-svgdx") {
        // Special case this fence type to display the XML input
        // prior to the rendered SVG output.
        inject_xml(events, content);
    }
    inject_svgdx(events, content, backend)?;
    if block_type.starts_with("svgdx-xml") {
        // Special case this fence type to display the XML input
        // prior to the rendered SVG output.
        inject_xml(events, content);
    }
    Ok(())
}

fn inject_xml(events: &mut Vec<Event>, content: &str) {
    events.push(Html(
        "\n\n<div style='overflow-x: auto; font-size: 0.9em;'>\n".into(),
    ));
    events.push(Start(Tag::CodeBlock(CodeBlockKind::Fenced("xml".into()))));
    events.push(Text(content.to_owned().into()));
    events.push(End(TagEnd::CodeBlock));
    events.push(Html("\n</div>\n".into()));
}

fn inject_svgdx(events: &mut Vec<Event>, content: &str, backend: &SvgdxBackend) -> Result<()> {
    events.push(Start(Tag::Paragraph));
    // Need to avoid blank lines in the rendered SVG, as they can cause
    // markdown to resume 'normal' md processing, especially when e.g.
    // indentation can cause an implicit code block to be started.
    // See https://talk.commonmark.org/t/inline-html-breaks-when-using-indentation/3317
    // and https://spec.commonmark.org/0.31.2/#html-blocks
    let svg_output = backend
        .render(content)?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    events.push(Html(svg_output.into()));
    events.push(End(TagEnd::Paragraph));
    Ok(())
}

#[cfg(feature = "svgdx-builtin")]
fn render_builtin_svgdx(s: &str) -> String {
    let cfg = svgdx::TransformConfig {
        svg_style: Some(SVGDX_STYLE.to_string()),
        auto_style_mode: AutoStyleMode::Inline,
        scale: SVGDX_SCALE.parse().expect("valid svgdx scale"),
        ..Default::default()
    };
    svgdx::transform_str(s.to_string(), &cfg).unwrap_or_else(|e| svgdx_error_html(&e.to_string()))
}

fn render_external_svgdx(program: &OsString, input: &str) -> Result<String> {
    let mut child = Command::new(program)
        .arg("--scale")
        .arg(SVGDX_SCALE)
        .arg("--auto-style-mode")
        .arg(SVGDX_AUTO_STYLE_MODE)
        .arg("--svg-style")
        .arg(SVGDX_STYLE)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| {
            Error::msg(format!(
                "Failed to start external svgdx binary from environment variable {SVGDX_BIN_ENV_VAR:?} ({:?}): {err}",
                program
            ))
        })?;

    // Assumes svgdx reads entire content into memory to avoid deadlock;
    // if that changes we'd need a separate thread to read stdout / stderr
    // while writing to stdin.
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes()).map_err(|err| {
            Error::msg(format!(
                "Failed to write input to external svgdx binary from environment variable {SVGDX_BIN_ENV_VAR:?} ({:?}): {err}",
                program
            ))
        })?;
    }

    let output = child.wait_with_output().map_err(|err| {
        Error::msg(format!(
            "Failed while waiting for external svgdx binary from environment variable {SVGDX_BIN_ENV_VAR:?} ({:?}): {err}",
            program
        ))
    })?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    let message = if stderr.is_empty() {
        format!("svgdx exited with status {}", output.status)
    } else {
        stderr
    };
    Ok(svgdx_error_html(&message))
}

fn svgdx_error_html(message: &str) -> String {
    format!(
        r#"<div style="color: red; border: 5px double red; padding: 1em;">{}</div>"#,
        message.replace('\n', "<br/>")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    fn unique_test_path(ext: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        std::env::temp_dir().join(format!("mdbook-svgdx-test-{}-{nanos}.{ext}", process::id()))
    }

    struct TempFile {
        path: PathBuf,
    }

    impl TempFile {
        fn new(path: PathBuf) -> Self {
            Self { path }
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    #[cfg(unix)]
    fn create_svgdx_filter_script() -> TempFile {
        let path = unique_test_path("sh");
        fs::write(&path, "#!/bin/sh\nsed 's/width=\"20\"/width=\"21\"/'\n")
            .expect("write test script");
        let mut perms = fs::metadata(&path).expect("script metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).expect("set script permissions");
        TempFile::new(path)
    }

    // these tests should run with or without the svgdx-builtin feature
    mod external_tests {
        use super::*;
        use assertables::assert_contains;

        #[cfg(unix)]
        #[test]
        fn process_svgdx_with_external_backend() {
            let script = create_svgdx_filter_script();
            let content = r##"
```svgdx
<svg>
  <rect width="20" height="5"/>
</svg>
```
"##;

            let backend = SvgdxBackend::External(script.path.clone().into_os_string());
            let chapter = Chapter::new("test", content.to_owned(), ".", Vec::new());
            let result = codeblock_parser(&chapter, &backend).unwrap();

            assert_contains!(result, "class='svgdx'>");
            // The filter script should have replaced width="20" with width="21"
            assert_contains!(result, "<rect width=\"21\" height=\"5\"/>");
        }

        #[test]
        fn empty_external_env_var_is_rejected() {
            let err = SvgdxBackend::resolve(|| Some(OsString::new())).unwrap_err();

            assert_contains!(err.to_string(), SVGDX_BIN_ENV_VAR);
            assert_contains!(err.to_string(), "is set but empty");
        }

        #[test]
        fn missing_external_binary_reports_spawn_error() {
            let backend = SvgdxBackend::External(unique_test_path("missing").into_os_string());

            let err = backend.render("<svg/>").unwrap_err();

            assert_contains!(err.to_string(), "Failed to start external svgdx binary");
            assert_contains!(err.to_string(), SVGDX_BIN_ENV_VAR);
        }
    }

    #[cfg(feature = "svgdx-builtin")]
    mod builtin_tests {
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

<div style="##;
            let expected2 = r##" class='svgdx'>


<svg "##;
            let expected3 = r##"<rect width="20" height="5""##;
            let chapter = Chapter::new("test", content.to_owned(), ".", Vec::new());
            let result = codeblock_parser(&chapter, &SvgdxBackend::Builtin).unwrap();
            assert_contains!(result, expected1);
            assert_contains!(result, expected2);
            assert_contains!(result, expected3);

            let mut z = Book::new();
            z.push_item(chapter);
        }

        #[test]
        fn process_with_crlf() {
            // crlf-separated text seems to be parsed into multiple Text events;
            // check the fenced code block is still processed as a single unit.
            let content = [
                "Some **markdown** text",
                "",
                "```svgdx",
                "<svg>",
                r#"  <rect wh="20 5"/>"#,
                r#"  <rect xy="^|h" wh="20 5"/>"#,
                "</svg>",
                "```",
            ]
            .join("\r\n");

            let expected1 = r##"Some **markdown** text

<div style="##;
            let expected2 = r##" class='svgdx'>


<svg "##;
            let expected3 = r##"<rect x="20" y="0" width="20" height="5""##;
            let chapter = Chapter::new("test", content.to_owned(), ".", Vec::new());
            let result = codeblock_parser(&chapter, &SvgdxBackend::Builtin).unwrap();
            assert_contains!(result, expected1);
            assert_contains!(result, expected2);
            assert_contains!(result, expected3);

            let mut z = Book::new();
            z.push_item(chapter);
        }

        #[test]
        fn missing_external_env_var_falls_back_to_builtin() {
            let backend = SvgdxBackend::resolve(|| None).unwrap();
            assert_eq!(backend, SvgdxBackend::Builtin);
        }
    }

    #[cfg(not(feature = "svgdx-builtin"))]
    mod no_builtin_tests {
        use super::*;
        use assertables::assert_contains;

        #[test]
        fn missing_external_env_var_without_builtin_errors() {
            let err = SvgdxBackend::resolve(|| None).unwrap_err();

            assert_contains!(err.to_string(), SVGDX_BIN_ENV_VAR);
            assert_contains!(err.to_string(), "No svgdx backend is available");
            assert_contains!(err.to_string(), "svgdx-builtin");
        }
    }
}
