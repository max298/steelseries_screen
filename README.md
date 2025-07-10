# steelseries_screen

**An embedded-graphics driver for SteelSeries displays.**

This Rust crate allows you to draw graphics and text on supported SteelSeries keyboard displays using the [`embedded-graphics`](https://crates.io/crates/embedded-graphics) ecosystem.
Please note that this is a very early (hacky) implementation which is not finished at all.

The implementation uses the official GameSense-SDK, which limits the platforms to macOS and windows, as these are the only platforms
where the SteelSeries GG Client runs.

---

## Getting Started

Add the crate to your `Cargo.toml` (once published):

```toml
[dependencies]
steelseries-screen = { git = "https://github.com/max298/steelseries_screen" }
```

### Example
```rust
// initialize a display with the correct size/type of hardware
let mut display = SteelSeriesDisplay::new(
    steelseries_screen::SteelSeriesLCDType::Apex,
    // a game-name is required:
    "HELLO_WORLD".to_string(),
);
// the GameSense API expects the 'game' (or application) to register before it sends data
display.register();
// after registration we also need to bind the event we're going to send
display.bind();

let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
Text::new("Hello World!", Point::new(0, 6), text_style).draw(&mut display);
// call flush to send the current framebuffer to the device
display.flush();
```
