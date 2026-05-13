# `mps3-an536` examples

An RTIC example intended for QEMU's emulation of the Arm MPS3-AN536 evaluation board. This is
a system with two Cortex-R52 processors (Armv8-R AArch32 architecture), however we only
use one of them.

## Dependencies

#### 1. `qemu-run`

```console
$ cargo install qemu-run
```

#### 2. QEMU

Visit <https://www.qemu.org/> and install QEMU 9 or higher.

## Run

Run:

```bash
rustup target add armv8r-none-eabihf
cargo run --releaser
```

The `qemu-run` program will spawn `qemu-system-arm` with the correct options. It
will also gather defmt logs over the semihosting interface (which is mercifully
fast in QEMU, unlike on real hardware) and decode and render them to the screen.

```text
$ cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running `qemu-run --machine mps3-an536 --cpu cortex-r52 target/armv8r-none-eabihf/debug/mps3-an536`
00:00:00.001336 INFO  In entry point
00:00:00.004376 INFO  In init...
00:00:00.006090 INFO  ********* 2 second tick
00:00:00.006788 INFO  500 ms tick
00:00:00.508454 INFO  500 ms tick
00:00:01.013732 INFO  500 ms tick
```
