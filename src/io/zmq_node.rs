use serde::Serialize;
use serde::de::DeserializeOwned;
use bincode::{serialize, deserialize};
use io::zmq;
use prelude::*;

create_node!(
    ZMQSend<T>: (),
    [socket: zmq::Socket, flags: i32],
    [recv: Vec<T>],
    |node: &mut ZMQSend<T>, mut data: Vec<T>| {
        node.send(&mut data);
    },
    T: Serialize + Clone,
);

impl<T> ZMQSend<T>
where
    T: Serialize + Clone,
{
    pub fn send(&mut self, data: &mut Vec<T>) {
        let buffer: Vec<u8> = serialize(&data).unwrap();
        self.socket.send(&buffer, self.flags).unwrap();
    }
}

create_node!(
    ZMQRecv<T>: Vec<T>,
    [socket: zmq::Socket, flags: i32],
    [],
    |node: &mut ZMQRecv<T>| node.recv(),
    T: DeserializeOwned + Clone,
);

impl<T> ZMQRecv<T>
where
    T: DeserializeOwned + Clone,
{
    pub fn recv(&mut self) -> Vec<T> {
        println!("We got a thing");
        let bytes = self.socket.recv_bytes(self.flags).unwrap();
        let res: Vec<T> = deserialize(&bytes).unwrap();
        res
    }
}

pub fn zmq_recv<T>(socket_type: zmq::SocketType, flags: i32) -> ZMQRecv<T> 
where 
    T: DeserializeOwned + Clone,
{
    let context = zmq::Context::new();
    let socket = context.socket(socket_type).unwrap();
    ZMQRecv::new(socket, flags)
}

pub fn zmq_send<T>(socket_type: zmq::SocketType, flags: i32) -> ZMQSend<T>
where
    T: Serialize + Clone,
{
    let context = zmq::Context::new();
    let socket = context.socket(socket_type).unwrap();
    ZMQSend::new(socket, flags)
}

#[cfg(test)]
mod test {
    use byteorder::{NativeEndian, ReadBytesExt};
    use io::zmq_node;
    use prelude::*;
    use std::io::Cursor;
    use std::thread;
    use io::zmq;

    #[test]
    fn test_zmq() {
        create_node!(DataGen: Vec<u32>, [], [], |_| vec![1, 2, 3, 4, 5],);
        let mut data_node = DataGen::new();
        let mut zmq_send = zmq_node::zmq_send::<u32>(zmq::SocketType::PUB, 0);
        let mut zmq_recv = zmq_node::zmq_recv(zmq::SocketType::SUB, 0);
        create_node!(CheckNode: (), [], [recv: Vec<u8>], |_, data: Vec<u8>| {
            let mut cursor = Cursor::new(data);
            let mut dst = [0; 5];
            cursor.read_u32_into::<NativeEndian>(&mut dst).unwrap();
            assert_eq!(dst, [1, 2, 3, 4, 5]);
        });
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
