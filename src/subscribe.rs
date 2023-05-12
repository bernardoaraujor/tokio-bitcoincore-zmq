use crate::{error::Result, message::Message};
use std::{
    sync::mpsc::{channel, Receiver},
    thread,
};
use zmq::{Context, Socket};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Continue,
    Stop,
}

impl Action {
    #[inline]
    pub fn stop_if(cond: bool) -> Self {
        if cond {
            Self::Stop
        } else {
            Self::Continue
        }
    }

    #[inline]
    pub fn stop_if_err<T, E>(res: Result<T, E>) -> Self {
        Self::stop_if(res.is_err())
    }

    #[inline]
    pub fn continue_if(cond: bool) -> Self {
        Self::stop_if(!cond)
    }
}

/// Subscribes to a single ZMQ endpoint and returns a [`Receiver`]
#[inline]
pub fn subscribe_single(endpoint: &str) -> Result<Receiver<Result<Message>>> {
    let (tx, rx) = channel();
    let context = Context::new();

    let socket = new_socket_internal(&context, endpoint)?;

    thread::spawn(move || subscribe_internal(socket, |msg| Action::stop_if_err(tx.send(msg))));

    Ok(rx)
}

/// Subscribes to multiple ZMQ endpoints and returns a [`Receiver`]
#[inline]
pub fn subscribe_multi(endpoints: &[&str]) -> Result<Receiver<Result<Message>>> {
    let (tx, rx) = channel();
    let context = Context::new();

    for endpoint in endpoints {
        let tx = tx.clone();

        let socket = new_socket_internal(&context, endpoint)?;

        thread::spawn(move || subscribe_internal(socket, |msg| Action::stop_if_err(tx.send(msg))));
    }

    Ok(rx)
}

/// Subscribes to a single ZMQ endpoint and blocks the thread until [`Action::Stop`] is returned by the callback
#[inline]
pub fn subscribe_single_blocking<F: Fn(Result<Message>) -> Action>(
    endpoint: &str,
    callback: F,
) -> Result<()> {
    let context = Context::new();

    let socket = new_socket_internal(&context, endpoint)?;

    subscribe_internal(socket, callback);

    Ok(())
}

/// Subscribes to multiple ZMQ endpoints and blocks the thread until [`Action::Stop`] is returned by the callback
#[inline]
pub fn subscribe_multi_blocking<F: Fn(Result<Message>) -> Action>(
    endpoints: &[&str],
    callback: F,
) -> Result<()> {
    let (tx, rx) = channel();
    let context = Context::new();

    for endpoint in endpoints {
        let tx = tx.clone();

        let socket = new_socket_internal(&context, endpoint)?;

        thread::spawn(move || subscribe_internal(socket, |msg| Action::stop_if_err(tx.send(msg))));
    }

    for msg in rx {
        if callback(msg) == Action::Stop {
            break;
        }
    }

    Ok(())
}

#[inline]
fn new_socket_internal(context: &Context, endpoint: &str) -> Result<Socket, zmq::Error> {
    let socket = context.socket(zmq::SUB)?;
    socket.connect(endpoint)?;
    socket.set_subscribe(b"")?;

    Ok(socket)
}

#[inline]
fn subscribe_internal<F: Fn(Result<Message>) -> Action>(socket: Socket, callback: F) {
    loop {
        let msg = socket
            .recv_multipart(0)
            .map_err(|err| err.into())
            .and_then(|mp| mp.try_into());

        if callback(msg) == Action::Stop {
            break;
        }
    }
}

#[inline]
#[deprecated(
    since = "1.0.5",
    note = "this function was renamed to `subscribe_multi`"
)]
pub fn sub_zmq(endpoints: &[&str]) -> Result<Receiver<Result<Message>>> {
    subscribe_multi(endpoints)
}
