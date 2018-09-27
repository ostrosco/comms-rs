//! ========
//! This node implements a PRNS generator using an linear-feedback shift register
//! (LFSR).  These are often used in communications systems for various tasks which
//! require a merely pseudorandom sequence as they are very cheap and easy to
//! implement in hardware with the use of a single LFSR.  These tasks can include
//! things such as frequency hopping and spread spectrum waveform spreading codes.
//! As usual,
//! [the Wiki](https://en.wikipedia.org/wiki/Linear-feedback_shift_register#Fibonacci_LFSRs)
//! is an excellent reference for farther details, particularly regarding what
//! exactly the polynomial bitmask is all about.  Note that the implementation of
//! an LFSR in this code has the register shifting to the left rather than right.
//! This is different than the Wiki graphics, and I chose it because it matches the
//! feedback polynomials in standard form better in my opinion.
//! A final note regarding the arguments to the constructor: be careful to size the
//! input type as the type with the desired LFSR length.  If you simply do something
//! like `let mut node = prns(0xC0, 1);` you'll get a 32 bit LFSR, which may not be
//! what you want.  Doing `let mut node = prns(0xC0 as u8, 1);` indicates to the
//! node internals that you want an 8 bit LSFR implementation.
//! ### Example Useage
//! ```rust,no_run
//! use prn::prn_node;
//! let mut node = prns(0xC0 as u8, 0x01);
//! create_node!(
//!     CheckNode: (),
//!     [state: Vec<u8>],
//!     [recv: u8],
//!     |node: &mut CheckNode, x| if node.state.len() == 128 {
//!         assert_eq!(
//!             node.state,
//!             vec![
//!                 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0,
//!                 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1,
//!                 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 1,
//!                 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0,
//!                 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0,
//!                 0, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0,
//!                 0, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1,
//!                 0, 0
//!             ]
//!         );
//!     } else {
//!         node.state.push(x);
//!     }
//! );
//! let mut check_node = CheckNode::new(Vec::new());
//! connect_nodes!(mynode, check_node, recv);
//! start_nodes!(mynode);
//! let check = thread::spawn(move || {
//!     let now = Instant::now();
//!     loop {
//!         check_node.call();
//!         if now.elapsed().as_secs() > 1 {
//!             break;
//!         }
//!     }
//! });
//! assert!(check.join().is_ok());
//! ```

pub mod prn_node;
