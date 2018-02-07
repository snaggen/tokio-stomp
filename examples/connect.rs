#![allow(dead_code)]
#![feature(conservative_impl_trait)]

extern crate tokio_stomp;
extern crate tokio_core;
extern crate tokio_io;
extern crate futures;
#[macro_use]
extern crate failure;

use std::io::prelude::*;

use futures::prelude::*;
use tokio_io::AsyncRead;
use tokio_io::codec::Framed;
use tokio_stomp::*;
use tokio_core::net::TcpStream;
use futures::future::{ok as fok, err as ferr};

type Transport = Framed<TcpStream, StompCodec>;

fn main() {
    let mut reactor = tokio_core::reactor::Core::new().unwrap();
    let handle = reactor.handle();

    let addr = "127.0.0.1:61613".parse().unwrap();
    let tcp = TcpStream::connect(&addr, &handle)
        .map_err(|e| e.into())
        .and_then(|tcp| {
            let transport = tcp.framed(StompCodec);
            handshake(transport)
        })
        .and_then(|stream| {
            let msg = Stomp::Disconnect {
                receipt: None
            }.to_frame();
            stream.send(msg).map_err(|e| e.into())
        });
    reactor.run(tcp).unwrap();
}

fn handshake(transport: Transport) -> impl Future<Item=Transport, Error=failure::Error> {
    let msg = Stomp::Connect {
        accept_version: b"1.1,1.2",
        host: b"0.0.0.0",
        login: None,
        passcode: None,
        heartbeat: None
    }.to_frame();
    transport
        .send(msg)
        .and_then(|transport| transport.into_future()
                  .map_err(|(e, _)| e.into()))
        .and_then(|(frame, stream)| {
            let frame = frame.unwrap();
            let (_, frame) = parse_frame(&frame).unwrap();
            let msg = frame.to_stomp().unwrap();
            if let Stomp::Connected {..} = msg {
                fok(stream)
            } else {
                ferr(format_err!("unexpected reply"))
            }
        })
}