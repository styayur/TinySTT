//! TinySTT core library.

pub mod asr;
pub mod audio;
pub mod clipboard;
pub mod error;
pub mod hotkey;
pub mod log;
pub mod paste;
pub mod paths;
pub mod settings;
pub mod state;
pub mod tray;

#[cfg(feature = "native")]
pub mod app;
#[cfg(feature = "native")]
pub mod pipeline;
