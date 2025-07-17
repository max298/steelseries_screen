use std::io::Error;

use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};

/// SteelSeries-Devices which can be targeted
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
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

impl SteelSeriesLCDType {
    /// Helper to get all supported LCD Types
    pub fn all() -> &'static [SteelSeriesLCDType] {
        &[
            SteelSeriesLCDType::Apex,
            SteelSeriesLCDType::Arctis,
            SteelSeriesLCDType::GameDAC,
            SteelSeriesLCDType::Rival7x0,
        ]
    }

    /// returns the dimensions for each type of hardware
    pub fn dimensions(&self) -> Size {
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
    pub framebuffer: Vec<u8>,
}

impl SteelSeriesDisplay {
    /// Create a new instance of a display which can be used by embedded_graphics
    /// # Arguments
    ///
    /// * `lcd_type` - The device type which will be targeted
    ///
    pub fn new(lcd_type: SteelSeriesLCDType) -> SteelSeriesDisplay {
        let size = lcd_type.dimensions();

        let framebuffer = vec![0; size.width as usize * size.height as usize / 8];

        SteelSeriesDisplay {
            lcd_type,
            framebuffer,
        }
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
        let target = if color.is_on() { 255 } else { 0 };
        for i in 0..self.framebuffer.len() {
            self.framebuffer[i] = target;
        }
        Ok(())
    }

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let size = self.size();
        let width = i32::try_from(size.width).expect("Could not parse width!");
        let height = i32::try_from(size.height).expect("Could not parse height!");
        // fill the framebuffer
        for Pixel(coord, color) in pixels {
            let (x, y) = coord.into();
            if x >= 0 && y >= 0 && x < width && y <= height {
                let pixel_index: usize = usize::try_from(y).expect("Could not parse y-coord") * 128
                    + usize::try_from(x).expect("Could not parse x-coord!");
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
                println!("Warning: Ignoring attempt to draw pixel out of bounds! x: {x}, y: {y}");
            }
        }
        Ok(())
    }
}
