# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.59] - 2020-09-13

### Added

- We now support `@sha256:...`-style digests in image specifiers.
- We export a new `v2::ImageVersion` enumeration.

### Changed

- `Image::tag` has been replaced with `Image::version`.
- `Image::without_tag` has been replaced with `Image::without_version`.
