# `docker_compose-rs`: Support for working with `docker-compose.yml` files

**This is a work in progress!** I'm still implementing the basic file
format.

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
