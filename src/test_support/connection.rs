use std::cmp::min;
use std::pin::Pin;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use futures::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, BufReader, BufWriter, ReadBuf};

use crate::protocol::connection::Connection;
use crate::protocol::connection::{AsyncReader, AsyncWriter};
use std::marker::PhantomData;
use tokio::sync::Mutex;

impl<I, O> Connection<I, O>
where
    I: From<Vec<u8>>,
    O: Into<Vec<u8>>,
{
    pub fn with_channel() -> (Connection<I, O>, Sender<Vec<u8>>, Receiver<Vec<u8>>) {
        let (input_sender, input_receiver) = mpsc::channel::<Vec<u8>>();
        let (output_sender, output_receiver) = mpsc::channel::<Vec<u8>>();
        let connection = Self {
            input: Mutex::new(BufReader::new(Box::new(FakeTcpReader {
                input: input_receiver,
            }))),
            output: Mutex::new(BufWriter::new(Box::new(FakeTcpWriter {
                output: output_sender,
            }))),
            input_frame: PhantomData,
            output_frame: PhantomData,
        };
        (connection, input_sender, output_receiver)
    }
}

// READ

// type FakeReadStream = Vec<u8>;

struct FakeTcpReader {
    input: Receiver<Vec<u8>>,
}

impl AsyncRead for FakeTcpReader {
    fn poll_read(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let msg = self.input.recv().unwrap();
        let size: usize = min(msg.len(), buf.capacity());
        buf.put_slice(&msg[..size]);
        Poll::Ready(Ok(()))
    }
}

impl AsyncReader for FakeTcpReader {}

// WRITE

struct FakeTcpWriter {
    output: Sender<Vec<u8>>,
}

impl AsyncWriter for FakeTcpWriter {}

impl AsyncWrite for FakeTcpWriter {
    fn poll_write(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        self.output.send(buf.to_vec()).unwrap();
        return Poll::Ready(Ok(buf.len()));
    }
    fn poll_flush(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }
    fn poll_shutdown(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }
}
