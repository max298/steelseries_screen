use std::io::Error;

use crate::api::GameSenseAPI;
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};

/// SteelSeries-Devices which can be targeted
pub enum SteelSeriesLCDType {
    /// Rival 700 and Rival 710
    Rival7x0,
    /// Apex7, Apex 7 TKL, Apex Pro and Apex Pro TKL
    Apex,
    /// Arctis Pro Wireless
    Arctis,
    /// GameDAC or Arctis Pro
    GameDAC,
}

/// returns the dimensions for each type of hardware
impl SteelSeriesLCDType {
    fn dimensions(&self) -> Size {
        match self {
            Self::Apex => Size::new(128, 40),
            Self::Arctis => Size::new(128, 48),
            Self::GameDAC => Size::new(128, 52),
            Self::Rival7x0 => Size::new(128, 36),
        }
    }
}

/// Display driver for SteelSeries devices
pub struct SteelSeriesDisplay {
    lcd_type: SteelSeriesLCDType,
    framebuffer: Vec<u8>,
    api: GameSenseAPI,
}

impl SteelSeriesDisplay {
    /// Create a new instance of a display
    /// # Arguments
    ///
    /// * `lcd_type` - The device type which will be targeted
    /// * `game_name` - A game name which will be shown in the SteelSeries Desktop Application. Allowed are upper-case A-Z, 0-9, hyphen, and underscore.\
    /// * `game_description` - A description which will be shown in the SteelSeries Desktop Application
    /// * `developer` - A name of the developer, will also be shown in the SteelSeries Desktop Application
    pub fn new(lcd_type: SteelSeriesLCDType, game_name: String) -> SteelSeriesDisplay {
        let size = lcd_type.dimensions();
        let api = GameSenseAPI::new(game_name, size.width as u8, size.height as u8);

        let framebuffer = vec![0; size.width as usize * size.height as usize / 8];

        SteelSeriesDisplay {
            lcd_type,
            framebuffer,
            api,
        }
    }

    /// Set the game developer name. **Note**: Must be called prior to calling register()
    pub fn developer(&mut self, developer: &str) {
        self.api.developer(developer.to_string());
    }

    /// Set the game description name. **Note**: Must be called prior to calling register()
    pub fn game_description(&mut self, description: &str) {
        self.api.game_description(description.to_string());
    }

    /// Before sending data to the API, we must register each "game"
    pub fn register(&self) {
        let _ = self.api.register();
    }

    /// Before sending data to the API, we bind the event which will handle our updated data.
    pub fn bind(&self) {
        let _ = self.api.bind_event();
    }

    /// Call flush to actually send the data to the device. When calling flush, a POST to the
    /// API endpoint is being made, which will then update the device. Without calling flush,
    /// no data is being sent to the device.
    pub fn flush(&mut self) -> Result<(), Error> {
        let _ = self.api.send_event(&self.framebuffer);
        Ok(())
    }
}

impl OriginDimensions for SteelSeriesDisplay {
    fn size(&self) -> Size {
        self.lcd_type.dimensions()
    }
}

impl DrawTarget for SteelSeriesDisplay {
    type Color = BinaryColor;
    type Error = Error;

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let target = match color.is_on() {
            true => 255,
            false => 0,
        };
        for i in 0..self.framebuffer.len() {
            self.framebuffer[i] = target;
        }
        Ok(())
    }

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let width = self.size().width as i32;
        let height = self.size().height as i32;
        // fill the framebuffer
        for Pixel(coord, color) in pixels.into_iter() {
            let (x, y) = coord.into();
            if x >= 0 && y >= 0 && x < width && y <= height {
                let pixel_index: usize = y as usize * 128 + x as usize;
                let byte_index: usize = pixel_index / 8;
                let bit_offset = 7 - (pixel_index % 8); // MSB-first in each byte
                match color {
                    BinaryColor::On => {
                        self.framebuffer[byte_index] |= 1 << bit_offset;
                    }
                    BinaryColor::Off => {
                        self.framebuffer[byte_index] &= !(1 << bit_offset);
                    }
                }
            } else {
                println!("Warning: Ignoring attempt to draw out of bounds!");
            }
        }
        Ok(())
    }
}
