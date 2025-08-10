# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.0 2025-08-10]

- Changed: updated to svgdx 0.22.1

## [0.8.1 2025-07-17]

- Fixed: handle fenced code block split across Text events (#1)

## [0.8.0 2025-06-29]

- Changed: updated to svgdx 0.21.0

## [0.7.0 2025-04-22]

- Changed: updated to svgdx 0.20.0

## [0.6.0 2025-03-01]

- Changed: updated to svgdx 0.19.0 (Note: minor breaking format changes)

## [0.5.0 2025-02-02]

- Changed: updated to svgdx 0.18.0 (Note: breaking format changes)

## [0.4.0 2024-12-31]

- Changed: updated to svgdx 0.16.0

- Changed: style changes, including `max-width:100%` on SVG images and use of the
  `use-local-styles` option of svgdx.

- Added: `xml-svgdx` in addition to `svgdx-xml`, as well as `-inline` variants of each
  to support additional display formats for diagrams and corresponding svgdx source.

## [0.3.0 - 2024-11-18]

- Changed: updated to svgdx 0.14.0

- Fixed: indentation and newlines in rendered SVG could break markdown processing

- Added: `svgdx-xml` fenced code block type to display both XML and SVG output

- Added: wrap rendered svgdx blocks in divs with fenced code block type for CSS styling

## [0.2.0 - 2024-10-03]

- Changed: updated to svgdx 0.13.0

- Added: `--version` option also displays underlying svgdx version

## [0.1.0 - 2024-09-25]

- Initial release using svgdx 0.12.0
