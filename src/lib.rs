//! Display driver for SteelSeries GG
//!
//! This crate provides a very basic driver for the SteelSeries LCD Displays.
//! It uses the official GameSense SDK which is a framework in SteelSeries Engine
//! that allows games & apps to send status updates to Engine.
//! It only supports LCD things and as it uses the official SteelSeries Engine,
//! it only works for macOS and windows and it is required to have the SteelSeries
//! GG Application running.

mod api;
mod display;

pub use crate::display::SteelSeriesDisplay;
pub use crate::display::SteelSeriesLCDType;
