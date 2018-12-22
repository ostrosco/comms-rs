use bincode::{deserialize, serialize};
use crate::io::zmq;
use crate::prelude::*;
use serde::de::DeserializeOwned;
use serde::Serialize;

create_node!(
    #[doc = "A node that will send serialized data out a ZMQ socket."]
    ZMQSend<T>: (),
    [socket: zmq::Socket, flags: i32],
    [recv: T],
    |node: &mut ZMQSend<T>, mut data: T| {
        node.send(&mut data)
    },
    T: Serialize + Clone,
);

impl<T> ZMQSend<T>
where
    T: Serialize + Clone,
{
    pub fn send(&mut self, data: &mut T) -> Result<(), NodeError> {
        let buffer: Vec<u8> = match serialize(&data) {
            Ok(b) => b,
            Err(_) => return Err(NodeError::DataError),
        };
        match self.socket.send(&buffer, self.flags) {
            Ok(_) => Ok(()),
            Err(_) => Err(NodeError::CommError),
        }
    }
}

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
/// let mut rand = rand_node::normal(0.0, 1.0);
/// let mut send: ZMQSend<f64> = zmq_node::zmq_send("tcp://*:5556",
///     zmq::SocketType::PUB, 0);
/// connect_nodes!(rand, send, recv);
/// start_nodes!(rand, send);
/// # }
pub fn zmq_send<T>(
    endpoint: &str,
    socket_type: zmq::SocketType,
    flags: i32,
) -> ZMQSend<T>
where
    T: Serialize + Clone,
{
    let context = zmq::Context::new();
    let socket = context.socket(socket_type).unwrap();
    socket.bind(endpoint).unwrap();
    ZMQSend::new(socket, flags)
}

create_node!(
    #[doc = "A node that will receive serialized data from a ZMQ socket."]
    ZMQRecv<T>: T,
    [socket: zmq::Socket, flags: i32],
    [],
    |node: &mut ZMQRecv<T>| node.recv(),
    T: DeserializeOwned + Clone,
);

impl<T> ZMQRecv<T>
where
    T: DeserializeOwned + Clone,
{
    pub fn recv(&mut self) -> Result<T, NodeError> {
        let bytes = match self.socket.recv_bytes(self.flags) {
            Ok(b) => b,
            Err(_) => return Err(NodeError::CommError),
        };
        let res: T = match deserialize(&bytes) {
            Ok(r) => r,
            Err(_) => return Err(NodeError::DataError),
        };
        Ok(res)
    }
}

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
/// let mut recv: ZMQRecv<Vec<Complex<u32>>> = zmq_node::zmq_recv(
///     "tcp://localhost:5556",
///     zmq::SocketType::SUB,
///     0);
/// let mut fft: FFTBatchNode<u32> = fft_node::fft_batch_node(1024, false);
/// connect_nodes!(recv, fft, recv);
/// start_nodes!(recv, fft);
/// # }
pub fn zmq_recv<T>(
    endpoint: &str,
    socket_type: zmq::SocketType,
    flags: i32,
) -> ZMQRecv<T>
where
    T: DeserializeOwned + Clone,
{
    let context = zmq::Context::new();
    let socket = context.socket(socket_type).unwrap();
    socket.connect(endpoint).unwrap();
    socket.set_subscribe(&[]).unwrap();
    ZMQRecv::new(socket, flags)
}

#[cfg(test)]
mod test {
    use crate::io::zmq;
    use crate::io::zmq_node;
    use crate::prelude::*;
    use std::thread;

    #[test]
    fn test_zmq() {
        create_node!(DataGen: Vec<u32>, [], [], |_| -> Result<
            Vec<u32>,
            NodeError,
        > {
            Ok(vec![1, 2, 3, 4, 5])
        });
        let mut data_node = DataGen::new();
        let mut zmq_send = zmq_node::zmq_send::<Vec<u32>>(
            "tcp://*:5556",
            zmq::SocketType::PUB,
            0,
        );
        let mut zmq_recv = zmq_node::zmq_recv::<Vec<u32>>(
            "tcp://localhost:5556",
            zmq::SocketType::SUB,
            0,
        );
        create_node!(CheckNode: (), [], [recv: Vec<u32>], |_,
                                                           data: Vec<u32>|
         -> Result<
            (),
            NodeError,
        > {
            assert_eq!(&data, &[1, 2, 3, 4, 5]);
            Ok(())
        });
        let mut check_node = CheckNode::new();
        connect_nodes!(data_node, zmq_send, recv);
        connect_nodes!(zmq_recv, check_node, recv);
        start_nodes!(data_node, zmq_send, zmq_recv);

        let handle = thread::spawn(move || {
            check_node.call().unwrap();
        });

        assert!(handle.join().is_ok());
    }
}
