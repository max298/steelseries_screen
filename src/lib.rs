#![warn(clippy::pedantic)]
//! Display driver for `SteelSeries GG`-LCD-Devices
//!
//! This crate provides a very basic driver for the SteelSeries LCD Displays.
//! It uses the official GameSense SDK which is a framework in SteelSeries Engine
//! that allows games & apps to send status updates to Engine.
//! It only supports LCD-related things and as it uses the official GameSense API,
//! thus it only works for macOS and windows and it is required to have the SteelSeries
//! GG Application running.

mod api;
mod display;

pub use crate::api::GameSenseAPI;
