# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0](https://github.com/doom-fish/screencapturekit-rs/compare/v1.0.0...v2.0.0) - 2025-11-27

### Added

- [**breaking**] add SCStreamConfigurationBuilder for consistent builder pattern

### Fixed

- redirect Swift build output to OUT_DIR for cargo publish
- revert manual version bump, let release-plz handle versioning
- bump version to 1.0.2
- exclude swift-bridge/.build from cargo package
- update preserves_aspect_ratio doctest for macOS 13 compatibility
- ignore Apple framework leaks in leak test
- correct Swift FFI function names to match Rust declarations
- rewrite ShareableContent.swift for Swift 5 concurrency
- add compiler guards for macOS 15 microphone APIs
- add compiler guards for macOS 15.0+ APIs
- update repository URLs to doom-fish/screencapturekit-rs

### Other

- reset version to 1.0.0 for release-plz
- *(examples)* use new builder() API instead of deprecated build()
- fix builder pattern description in README
- fix README inconsistencies with actual API
- add contributors section to README
- improve Cargo.toml metadata
- update README badges for doom-fish repo and GitHub Pages docs
- add GitHub Pages documentation hosting
- add --git-token flag to release-plz commands
- cache cargo registry and release-plz binary
- add macOS 26 to build matrix
- add build matrix for macOS 13/14/15 (Intel + ARM64)
- use macos-latest for release-plz (requires macOS frameworks)
- use CARGO_REGISTRY_TOKEN for publishing
- bump version to 1.0.1
- add release environment for trusted publishing

## [0.3.6](https://github.com/doom-fish/screencapturekit-rs/compare/v0.3.5...v0.3.6) - 2025-08-04

### Added

- Get and set sample rate via SCStreamConfiguration ([#94](https://github.com/doom-fish/screencapturekit-rs/pull/94))

### Other

- *(deps)* update core-graphics requirement from 0.24 to 0.25 ([#92](https://github.com/doom-fish/screencapturekit-rs/pull/92))
- workflows
- Update CHANGELOG.md
- Delete .github/workflows/contrib.yml

## [0.3.5](https://github.com/doom-fish/screencapturekit-rs/compare/v0.3.4...v0.3.5) - 2025-02-06

### Other

- fix releaze action
- fix bad cargo.toml

## [0.3.4](https://github.com/doom-fish/screencapturekit-rs/compare/v0.3.3...v0.3.4) - 2025-01-29

### Other

- chore(contributors) update contrib ([#80](https://github.com/doom-fish/screencapturekit-rs/pull/80))

## [0.3.3](https://github.com/doom-fish/screencapturekit-rs/compare/v0.3.2...v0.3.3) - 2025-01-29

### Added

- add showsCursor configuration option (#72)

### Other

- Fix scstream double free ([#74](https://github.com/doom-fish/screencapturekit-rs/pull/74))
- *(deps)* update block2 requirement from 0.5 to 0.6 (#75)

## [0.3.2](https://github.com/doom-fish/screencapturekit-rs/compare/v0.3.1...v0.3.2) - 2024-12-19

### Added

- add Send trait for SCShareableContent (#59)
- add screenshot manager capture (#58)
- add configuration options for captured frames (#57)

## [0.3.1](https://github.com/doom-fish/screencapturekit-rs/compare/v0.3.0...v0.3.1) - 2024-11-29

### Other

- remove old changelog
