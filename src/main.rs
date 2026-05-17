use clap::{Arg, ArgMatches, Command};
use mdbook_preprocessor::errors::Error;
use mdbook_preprocessor::{Preprocessor, parse_input};
use semver::{Version, VersionReq};
use std::process;
use std::{env, io};

use mdbook_svgdx::SvgdxProc;

#[cfg(feature = "svgdx-builtin")]
fn svgdx_version_label() -> String {
    svgdx::VERSION.into()
}

#[cfg(not(feature = "svgdx-builtin"))]
fn svgdx_version_label() -> String {
    use mdbook_svgdx::SVGDX_BIN_ENV_VAR;
    format!("via {SVGDX_BIN_ENV_VAR}")
}

fn make_app() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(format!(
            "{} (svgdx {}; mdbook {})",
            env!("CARGO_PKG_VERSION"),
            svgdx_version_label(),
            mdbook_preprocessor::MDBOOK_VERSION
        ))
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    let matches = make_app().get_matches();

    let preprocessor = SvgdxProc {};

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&preprocessor, sub_args);
    } else if let Err(e) = handle_preprocessing(&preprocessor) {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = parse_input(io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(mdbook_preprocessor::MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook_preprocessor::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> ! {
    let renderer = sub_args
        .get_one::<String>("renderer")
        .expect("Required argument");
    let supported = pre.supports_renderer(renderer);

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if let Ok(true) = supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
