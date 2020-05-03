//! Utility nodes that don't fit into any specific category and helper
//! functions for developing your own nodes.

use std::error;
use std::fmt;

#[derive(Clone, Debug)]
pub enum MathError {
    ConvertError,
    InvalidRolloffError,
}

impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let desc = match *self {
            MathError::ConvertError => "Type conversion from generic failed",
            MathError::InvalidRolloffError => {
                "Invalid rolloff parameter, must be on interval [0.0, 1.0]"
            }
        };
        write!(f, "Math error: {}", desc)
    }
}

impl error::Error for MathError {
    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}

/// Some basic math functions used elsewhere in the project
pub mod math;
/// Node for creating an OpenGL window and plotting data
pub mod plot_node;
/// Some nodes to aid in the generation of random numbers
pub mod rand_node;
/// Some nodes to aid in resampling signals
pub mod resample_node;
