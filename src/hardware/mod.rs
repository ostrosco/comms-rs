//! Provides interfaces to hardware platforms and generic traits to encapsulate
//! radio functionality.
//!

#[cfg(feature = "rtlsdr_node")]
extern crate rtlsdr;

#[cfg(feature = "rtlsdr_node")]
pub mod rtlsdr_radio;

pub mod radio;
