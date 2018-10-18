use prelude::*;
use zmq::Socket;
use byteorder::ReadBytesExt;

create_node!(
    ZMQSend<T>: (),
    [socket: Socket, flags: i32],
    [recv: T],
    |node: &mut ZMQSend<T>, mut data: T| {
        node.send(&mut data);
    },
    T: ReadBytesExt,
);

impl<T> ZMQSend<T>
where
    T: ReadBytesExt
{
    pub fn send(&mut self, data: &mut T) {
        let mut buffer = vec![];
        data.read_to_end(&mut buffer).unwrap();
        self.socket.send(&buffer, self.flags).unwrap()
    }
}

create_node!(
    ZMQRecv: Vec<u8>,
    [socket: Socket, flags: i32],
    [],
    |node: &mut ZMQRecv| {
        node.recv()
    }
);

impl ZMQRecv
{
    pub fn recv(&mut self) -> Vec<u8> {
        self.socket.recv_bytes(self.flags).unwrap()
    }
}
