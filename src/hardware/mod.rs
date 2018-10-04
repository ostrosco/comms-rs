//! Provides interfaces to hardware platforms and generic traits to encapsulate
//! radio functionality.
//!

#[cfg(feature = "rtlsdr_support")]
extern crate rtlsdr;

#[cfg(feature = "rtlsdr_support")]
pub mod rtlsdr_radio;

pub mod radio;
