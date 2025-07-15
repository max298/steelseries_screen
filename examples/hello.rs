use std::time::Duration;

use embedded_graphics::{
    mono_font::{MonoTextStyle, iso_8859_10::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use steelseries_screen::GameSenseAPI;

fn main() {
    let mut api = GameSenseAPI::new("HELLO_WORLD".to_string());

    // optional: set developer name and game-description
    api.developer("Max".to_string());
    api.game_description("This shows hello world!".to_string());

    // the GameSense API expects the 'game' (or application) to register before it sends data
    let _ = api.register();
    // after registration we also need to bind the event we're going to send
    let _ = api.bind_event();
    // send a heartbeat every 10 seconds to prevent the display from being reset if no data is sent
    api.register_heartbeat();

    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let _ = Text::new("Hello World!", Point::new(0, 6), text_style).draw(api.display_apex_mut());

    // call update_displays to update the devices
    // note that in this example we only send data for a display of type apex (128x40).
    let _ = api.update_displays();

    // show hello world for 60 seconds
    std::thread::sleep(Duration::from_secs(60));
    api.unregister_heartbeat();
}
