use bincode::{deserialize, serialize};
use io::zmq;
use prelude::*;
use serde::de::DeserializeOwned;
use serde::Serialize;

create_node!(
    ZMQSend<T>: (),
    [socket: zmq::Socket, flags: i32],
    [recv: T],
    |node: &mut ZMQSend<T>, mut data: T| {
        node.send(&mut data);
    },
    T: Serialize + Clone,
);

impl<T> ZMQSend<T>
where
    T: Serialize + Clone,
{
    pub fn send(&mut self, data: &mut T) {
        let buffer: Vec<u8> = serialize(&data).unwrap();
        self.socket.send(&buffer, self.flags).unwrap();
    }
}

create_node!(
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
    pub fn recv(&mut self) -> T {
        let bytes = self.socket.recv_bytes(self.flags).unwrap();
        let res: T = deserialize(&bytes).unwrap();
        res
    }
}

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

#[cfg(test)]
mod test {
    use byteorder::{NativeEndian, ReadBytesExt};
    use io::zmq;
    use io::zmq_node;
    use prelude::*;
    use std::io::Cursor;
    use std::thread;

    #[test]
    fn test_zmq() {
        create_node!(DataGen: Vec<u32>, [], [], |_| vec![1, 2, 3, 4, 5],);
        let mut data_node = DataGen::new();
        let mut zmq_send =
            zmq_node::zmq_send::<Vec<u32>>("tcp://*:5556", zmq::SocketType::PUB, 0);
        let mut zmq_recv = zmq_node::zmq_recv::<Vec<u32>>(
            "tcp://localhost:5556",
            zmq::SocketType::SUB,
            0,
        );
        create_node!(
            CheckNode: (),
            [],
            [recv: Vec<u32>],
            |_, data: Vec<u32>| {
                assert_eq!(&data, &[1, 2, 3, 4, 5]);
            }
        );
        let mut check_node = CheckNode::new();
        connect_nodes!(data_node, zmq_send, recv);
        connect_nodes!(zmq_recv, check_node, recv);
        start_nodes!(data_node, zmq_send, zmq_recv);

        let handle = thread::spawn(move || {
            check_node.call();
        });

        assert!(handle.join().is_ok());
    }
}
