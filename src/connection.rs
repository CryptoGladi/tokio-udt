use crate::socket::{SocketType, UdtStatus};
use crate::udt::{SocketRef, Udt};
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ErrorKind, ReadBuf, Result};

pub struct UdtConnection {
    socket: SocketRef,
}

impl UdtConnection {
    pub(crate) fn new(socket: SocketRef) -> Self {
        Self { socket }
    }

    pub async fn connect(addr: SocketAddr) -> Result<Self> {
        let socket = {
            let mut udt = Udt::get().write().await;
            udt.new_socket(SocketType::Stream, 10)?.clone()
        };
        socket.connect(addr).await?;
        let connection = Self::new(socket.clone());
        loop {
            if connection.socket.status() == UdtStatus::Connected {
                break;
            }
            connection.socket.connect_notify.notified().await;
        }
        Ok(connection)
    }

    pub async fn send(&self, msg: &[u8]) -> Result<()> {
        self.socket.send(msg)
    }

    pub async fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        let nbytes = self.socket.recv(buf).await?;
        Ok(nbytes)
    }
}

impl AsyncRead for UdtConnection {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        match self.socket.poll_recv(buf) {
            Poll::Ready(_) => Poll::Ready(Ok(())),
            Poll::Pending => {
                let waker = cx.waker().clone();
                let socket = self.socket.clone();
                tokio::spawn(async move {
                    socket.rcv_notify.notified().await;
                    waker.wake();
                });
                Poll::Pending
            }
        }
    }
}

impl AsyncWrite for UdtConnection {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        let buf_len = buf.len();
        match self.socket.send(buf) {
            Ok(_) => Poll::Ready(Ok(buf_len)),
            Err(err) => match err.kind() {
                ErrorKind::OutOfMemory => {
                    let waker = cx.waker().clone();
                    let socket = self.socket.clone();
                    tokio::spawn(async move {
                        socket.ack_notify.notified().await;
                        waker.wake();
                    });
                    Poll::Pending
                }
                _ => Poll::Ready(Err(err)),
            },
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        // TODO: Best effort shutdown
        // match self.socket.close() {
        //     Ok(_) => Poll::Ready(Ok(())),
        //     Err(e) => Poll::Ready(Err(Error::new(ErrorKind::Other, e.err_msg))),
        // }
        Poll::Ready(Ok(()))
    }
}
