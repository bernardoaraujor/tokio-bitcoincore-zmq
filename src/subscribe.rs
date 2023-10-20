use crate::{
    error::Result,
    message::{Message, SEQUENCE_LEN, TOPIC_MAX_LEN},
    Error, DATA_MAX_LEN,
};
use core::{convert::Infallible, ops::ControlFlow};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use zmq::{Context, Socket};

/// Subscribes to multiple ZMQ endpoints and returns a [`Receiver`].
#[inline]
pub fn subscribe_multi(endpoints: &[&str]) -> Result<Receiver<Result<Message>>> {
    let (tx, rx) = channel(endpoints.len());
    let context = Context::new();

    for endpoint in endpoints {
        let tx = tx.clone();

        let socket = new_socket_internal(&context, endpoint)?;

        tokio::spawn(subscribe_internal::<()>(socket, tx));
    }

    Ok(rx)
}


/// Subscribes to a single ZMQ endpoint and returns a [`Receiver`].
#[inline]
pub async fn subscribe_single(endpoint: &str) -> Result<Receiver<Result<Message>>> {
    let (tx, rx) = channel(1);
    let context = Context::new();

    let socket = new_socket_internal(&context, endpoint)?;

    tokio::spawn(subscribe_internal::<()>(socket, tx));

    Ok(rx)
}

#[inline]
fn new_socket_internal(context: &Context, endpoint: &str) -> Result<Socket> {
    let socket = context.socket(zmq::SUB)?;
    socket.connect(endpoint)?;
    socket.set_subscribe(b"")?;

    Ok(socket)
}

#[inline]
fn recv_internal(socket: &Socket, data: &mut [u8; DATA_MAX_LEN]) -> Result<Message> {
    let mut topic = [0u8; TOPIC_MAX_LEN];
    let mut sequence = [0u8; SEQUENCE_LEN];

    let topic_len = socket.recv_into(&mut topic, 0)?;
    if topic_len > TOPIC_MAX_LEN {
        return Err(Error::InvalidTopic(topic_len, topic));
    }

    if !socket.get_rcvmore()? {
        return Err(Error::InvalidMutlipartLength(1));
    }

    let data_len = socket.recv_into(data, 0)?;
    if data_len > DATA_MAX_LEN {
        return Err(Error::InvalidDataLength(data_len));
    }

    if !socket.get_rcvmore()? {
        return Err(Error::InvalidMutlipartLength(2));
    }

    let sequence_len = socket.recv_into(&mut sequence, 0)?;
    if sequence_len != SEQUENCE_LEN {
        return Err(Error::InvalidSequenceLength(sequence_len));
    }

    if !socket.get_rcvmore()? {
        return Message::from_parts(&topic[0..topic_len], &data[0..data_len], sequence);
    }

    let mut len = 3;

    loop {
        socket.recv_into(&mut [], 0)?;

        len += 1;

        if !socket.get_rcvmore()? {
            return Err(Error::InvalidMutlipartLength(len));
        }
    }
}

#[inline]
async fn subscribe_internal<B>(
    socket: Socket,
    tx: Sender<Result<Message>>,
) -> ControlFlow<B, Infallible> {
    let mut data: Box<[u8; DATA_MAX_LEN]> =
        vec![0; DATA_MAX_LEN].into_boxed_slice().try_into().unwrap();

    loop {
        let msg = recv_internal(&socket, &mut data);

        match tx.send(msg).await {
            Ok(_) => ControlFlow::Continue(()),
            Err(_) => ControlFlow::Break(()),
        };
    }
}
