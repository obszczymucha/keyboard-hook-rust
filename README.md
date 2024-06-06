# keyboard-hook-rust

A keyboard mapping framework for Windows.


## Developing on Linux with cross-compilation for Windows

### Requirements
 * Neovim
 * folke/neoconf plugin
 * mingw-w64 package (`sudo pacman -Sy mingw-w64`)


### How to use
See `examples/demo.rs` and run it:
  ```bash
  cargo run --example demo
  ```

### Development pro-tip
When developing on **WSL2**, make sure to clone the project on the **Windows**
drive, and not inside **WSL2**. You might get a warning from **Windows** that
it's not recommended, because it's slow, but it's actually bullshit. In fact,
the reverse is true-if you clone it inside **WSL2**, then `cargo test` takes
forever, but it runs instantaneously on **Windows** drives.

Bottom line: **fuck Windows**.

