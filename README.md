# `docker_compose-rs`: Support for working with `docker-compose.yml` files

[![Latest version](https://img.shields.io/crates/v/docker_compose.svg)](https://crates.io/crates/docker_compose) [![License](https://img.shields.io/crates/l/docker_compose.svg)](https://creativecommons.org/publicdomain/zero/1.0/) [![Build Status](https://travis-ci.org/emk/docker_compose-rs.svg?branch=master)](https://travis-ci.org/emk/docker_compose-rs)

**This is a work in progress!** Most of `services:` is supported, but I'm
still refining the APIs as higher-level tools get build around this.

[API Documention](http://docs.randomhacks.net/docker_compose-rs/)

## Goals

`docker-compose.yml` is a very useful format, but it's hard to parse and
transform correctly.  This library aims to offer:

- High-level, type-safe APIs for anything you can find in a
  `docker-compose.yml` file.
- Parsing of individual string fields into real objects.
- Support for working with strings that might contain variable
  interpolations, and leaving them unparsed when necessary.
- Canonical representations of fields which may have multiple formats.
- Easy updates when `docker-compose.yml` gets extended.

## Building

You can build this library using stable Rust version 1.11.  But if you want
to develop it, you will get _much_ better error messages using a nightly
build of Rust.

```sh
# Install Rust stable and nightly using rustup.
curl -sSf https://static.rust-lang.org/rustup.sh | sh
rustup toolchain install nightly

# Build unit tests using nightly Rust.
rustup run nightly cargo test --no-default-features --features unstable
```

## Sponsor

<a href="http://www.faraday.io"><img
src="http://cdn2.hubspot.net/hubfs/515497/img/logo.svg" alt="Faraday
logo"/></a>

Part of the work on [`docker_compose-rs`][docker_compose-rs] has been
generously sponsored by [Faraday][] for use in
their [`conductor`][conductor] tool, which orchestrates `docker-compose`
for large, multi-pod apps.

[Faraday]: http://www.faraday.io/
[conductor]: https://github.com/faradayio/conductor
[docker_compose-rs]: https://github.com/emk/docker_compose-rs
