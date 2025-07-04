# Inspecting generated code

`#[rtic::app]` is a procedural macro that produces support code.
If for some reason you need to inspect the code generated by this macro you have two options:

* You can inspect the file `rtic-expansion.rs` inside the `target` directory.
* Use the [`cargo-expand`] sub-command

## Using generated `rtic-expansion.rs`

Locating this file depends on how building is performed.

Using e.g. `cargo xtask build-example` within the main RTIC repo will place the file based on "platform" used:
```
$ cargo xtask example-build --example smallest
$ cargo xtask example-build --example monotonic --platform esp32-c3

$ fd -u rtic-expansion.rs
examples/esp32c3/target/rtic-expansion.rs
examples/lm3s6965/target/rtic-expansion.rs
```

In the regular cargo project case it goes directly in the `target` folder.

This file contains the expansion of the `#[rtic::app]` item (not your whole program!) of the *last built* (via `cargo build` or `cargo check`) RTIC application.
The expanded code is not pretty printed by default, so you'll want to run `rustfmt` on it before you read it.

``` console
$ cargo build --example smallest --target thumbv7m-none-eabi
```

``` console
$ rustfmt target/rtic-expansion.rs
```

``` console
$ tail target/rtic-expansion.rs
```

``` rust,noplayground
#[doc = r" Implementation details"]
mod app {
    #[doc = r" Always include the device crate which contains the vector table"]
    use lm3s6965 as _;
    #[no_mangle]
    unsafe extern "C" fn main() -> ! {
        rtic::export::interrupt::disable();
        let mut core: rtic::export::Peripherals = core::mem::transmute(());
        core.SCB.scr.modify(|r| r | 1 << 1);
        rtic::export::interrupt::enable();
        loop {
            rtic::export::wfi()
        }
    }
}
```

## Using `cargo-expand` tool

If not available, install:

```
$ cargo install cargo-expand
```

This sub-command will expand *all* the macros, including the `#[rtic::app]` attribute, and modules in your crate and print the output to the console.


[`cargo-expand`]: https://crates.io/crates/cargo-expand

``` console
# produces the same output as before
```

``` console
cargo expand --example smallest | tail
```
