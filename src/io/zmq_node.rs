use crate::io::zmq;
use crate::prelude::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_cbor::de::from_slice;
use serde_cbor::ser::to_vec_packed;

/// A node that will send serialized data out of a ZMQ socket.
#[derive(Node)]
#[pass_by_ref]
pub struct ZMQSend<T>
where
    T: Serialize + Clone + Send,
{
    pub input: NodeReceiver<T>,
    socket: zmq::Socket,
    flags: i32,
}

impl<T> ZMQSend<T>
where
    T: Serialize + Clone + Send,
{
    /// Creates a node to serialize and send data out via ZeroMQ.
    ///
    /// Example:
    ///
    /// ```
    /// # #[macro_use] extern crate comms_rs;
    /// # extern crate zmq;
    /// # use comms_rs::prelude::*;
    /// # use comms_rs::io::zmq_node::{self, ZMQSend};
    /// # use comms_rs::util::rand_node;
    /// # fn main() {
    /// // Generate random numbers and broadcast them out via ZeroMQ.
    /// let mut rand = rand_node::NormalNode::new(0.0, 1.0);
    /// let mut send: ZMQSend<f64> = ZMQSend::new("tcp://*:5556",
    ///     zmq::SocketType::PUB, 0);
    /// connect_nodes!(rand, output, send, input);
    /// start_nodes!(rand, send);
    /// # }
    pub fn new(
        endpoint: &str,
        socket_type: zmq::SocketType,
        flags: i32,
    ) -> Self {
        let context = zmq::Context::new();
        let socket = context.socket(socket_type).unwrap();
        socket.bind(endpoint).unwrap();
        ZMQSend {
            socket,
            flags,
            input: Default::default(),
        }
    }

    pub fn run(&mut self, data: &T) -> Result<(), NodeError> {
        self.send(data)
    }

    pub fn send(&mut self, data: &T) -> Result<(), NodeError> {
        let buffer: Vec<u8> = match to_vec_packed(&data) {
            Ok(b) => b,
            Err(_) => return Err(NodeError::DataError),
        };
        match self.socket.send(&buffer, self.flags) {
            Ok(_) => Ok(()),
            Err(_) => Err(NodeError::CommError),
        }
    }
}

/// A node that will receiver serialized data from a ZMQ socket.
#[derive(Node)]
pub struct ZMQRecv<T>
where
    T: DeserializeOwned + Clone + Send,
{
    socket: zmq::Socket,
    flags: i32,
    pub output: NodeSender<T>,
}

impl<T> ZMQRecv<T>
where
    T: DeserializeOwned + Clone + Send,
{
    /// Creates a node to receive data from a ZMQ socket.
    ///
    /// Example:
    ///
    /// ```
    /// # #[macro_use] extern crate comms_rs;
    /// # extern crate zmq;
    /// # extern crate num;
    /// # use comms_rs::prelude::*;
    /// # use comms_rs::io::zmq_node::{self, ZMQRecv};
    /// # use comms_rs::fft::fft_node::{FFTBatchNode, self};
    /// # use num::Complex;
    /// # fn main() {
    /// // Generate random numbers and broadcast them out via ZeroMQ.
    /// let mut recv: ZMQRecv<Vec<Complex<u32>>> = ZMQRecv::new(
    ///     "tcp://localhost:5556",
    ///     zmq::SocketType::SUB,
    ///     0);
    /// let mut fft: FFTBatchNode<u32> = FFTBatchNode::new(1024, false);
    /// connect_nodes!(recv, output, fft, input);
    /// start_nodes!(recv, fft);
    /// # }
    pub fn new(
        endpoint: &str,
        socket_type: zmq::SocketType,
        flags: i32,
    ) -> Self {
        let context = zmq::Context::new();
        let socket = context.socket(socket_type).unwrap();
        socket.connect(endpoint).unwrap();
        socket.set_subscribe(&[]).unwrap();
        ZMQRecv {
            socket,
            flags,
            output: Default::default(),
        }
    }

    pub fn run(&mut self) -> Result<T, NodeError> {
        self.recv()
    }

    pub fn recv(&mut self) -> Result<T, NodeError> {
        let bytes = match self.socket.recv_bytes(self.flags) {
            Ok(b) => b,
            Err(_) => return Err(NodeError::CommError),
        };
        let res: T = match from_slice(&bytes) {
            Ok(r) => r,
            Err(_) => return Err(NodeError::DataError),
        };
        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use crate::io::zmq;
    use crate::io::zmq_node::{ZMQRecv, ZMQSend};
    use crate::prelude::*;
    use std::thread;

    #[test]
    fn test_zmq() {
        #[derive(Node)]
        struct DataGen {
            pub output: NodeSender<Vec<u32>>,
        }
        impl DataGen {
            pub fn new() -> Self {
                DataGen {
                    output: Default::default(),
                }
            }

            pub fn run(&mut self) -> Result<Vec<u32>, NodeError> {
                Ok(vec![1, 2, 3, 4, 5])
            }
        }

        let mut data_node = DataGen::new();
        let mut zmq_send: ZMQSend<Vec<u32>> =
            ZMQSend::new("tcp://*:5556", zmq::SocketType::PUB, 0);
        let mut zmq_recv: ZMQRecv<Vec<u32>> =
            ZMQRecv::new("tcp://localhost:5556", zmq::SocketType::SUB, 0);

        #[derive(Node)]
        #[pass_by_ref]
        struct CheckNode {
            pub input: NodeReceiver<Vec<u32>>,
        }

        impl CheckNode {
            pub fn new() -> Self {
                CheckNode {
                    input: Default::default(),
                }
            }

            pub fn run(&mut self, data: &[u32]) -> Result<(), NodeError> {
                assert_eq!(&data, &[1, 2, 3, 4, 5]);
                Ok(())
            }
        }
        let mut check_node = CheckNode::new();
        connect_nodes!(data_node, output, zmq_send, input);
        connect_nodes!(zmq_recv, output, check_node, input);
        start_nodes!(data_node, zmq_send, zmq_recv);

        let handle = thread::spawn(move || {
            check_node.call().unwrap();
        });

        assert!(handle.join().is_ok());
    }
}
