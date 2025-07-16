# steelseries_screen

**An embedded-graphics driver for SteelSeries displays.**

This Rust crate allows you to draw graphics and text on supported SteelSeries devices with displays using the [`embedded-graphics`](https://crates.io/crates/embedded-graphics) ecosystem.
Please note that this is a very early (hacky) implementation which is not finished yet, but it does work.

The implementation uses the official GameSense-SDK, which limits the platforms to macOS and windows, as these are the only platforms where the SteelSeries GG
Client runs. For Linux you can checkout the awesome [apex-tux](https://github.com/not-jan/apex-tux) project which also works on linux.

---

## Getting Started

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
steelseries_screen = { git = "https://github.com/max298/steelseries_screen" }
```

### Example
```rust
// initialize the API
let mut api = steelseries_screen::GameSenseAPI::new("TEST");
// the GameSense API expects the 'game' (or application) to register before it sends data
api.register();
// after registration we also need to bind the event we're going to send
api.bind_event();

// do the actual drawing - note that we only target a display for apex-keyboards. use `api.display_*_mut()` for other devices
let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
Text::new("Hello World!", Point::new(0, 6), text_style).draw(&mut api.display_apex_mut());

// update the displays
api.update_displays();
```
