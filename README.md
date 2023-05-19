# winit-test

`winit-test` provides a simple test harness for testing `winit` applications.

## How to Use

Add your test to `Cargo.toml` with the `harness = false` option. This will prevent Rust's default test harness from running your test.

```toml
[[test]]
name = "my_test"
path = "tests/my_test.rs"
harness = false
```

Then, in your test, use the `winit_test::main!` macro to run your tests. The tests must be functions that take an `EventLoopWindowTarget`.

```rust
use winit_test::winit::event_loop::EventLoopWindowTarget;

fn my_test(elwt: &EventLoopWindowTarget<()>) {
    // ...
}

fn other_test(elwt: &EventLoopWindowTarget<()>) {
    // ...
}

winit_test::main!(my_test, other_test);
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)
 * Zlib license ([LICENSE-ZLIB](LICENSE-ZLIB) or https://opensource.org/licenses/Zlib)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

