//! Nodes for filtering samples.
//!
//! There are two primary categories of digital filters in signal processing:
//!
//! * Finite Impulse Response (FIR) Filters
//! * Infinite Impulse Response (IIR) Filters
//!
//! FIR filters are feedforward based systems, meaning they can't become
//! unstable regardless of the input data.  This can be desireable when system
//! guaranteed behavior is important.  Another pro to FIR filters is that they
//! are easy to implement, have a linear phase response, and a constant and
//! predictable group delay over frequency.
//!
//! The primary disadvantage of FIR filters is that they can be computationally
//! inefficient when compared to equivalent IIR filters, although often times
//! this disadvantage is not particularly important with modern systems due to
//! the extremely high performance commonly and cheaply available.
//!
//! IIR filters are feedback based systems, and have all the caveats associated
//! with any feedback system.  If poorly designed they can be unstable and
//! unpredictable.  The phase and group delay responses are non-linear.
//!
//! With those drawbacks noted, a well designed IIR filter can be stable in all
//! but the most unlikely scenarios, and extremely efficient as well.  Many
//! times a design that requires an 81 tap FIR filter could only require 9 taps
//! from a well designed IIR filter alternative.
pub mod fir;
pub mod fir_node;
