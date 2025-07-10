use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::{iso_8859_10::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use steelseries_screen::SteelSeriesDisplay;

fn main() {
    let mut display = SteelSeriesDisplay::new(
        steelseries_screen::SteelSeriesLCDType::Apex,
        "HELLO_WORLD".to_string(),
    );
    // optional: set developer name and game-description
    display.developer("Max");
    display.game_description("This shows hello world!");

    // the GameSense API expects the 'game' (or application) to register before it sends data
    display.register();
    // after registration we also need to bind the event we're going to send
    display.bind();

    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    let _ = Text::new("Hello World!", Point::new(0, 6), text_style).draw(&mut display);
    // call flush to send the current framebuffer to the device
    let _ = display.flush();
}
