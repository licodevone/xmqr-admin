// SPDX-License-Identifier: MIT
use axum::serve::Listener;
use std::{
    future::Future,
    io,
    net::SocketAddr,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::{TcpListener, TcpStream},
    sync::{OwnedSemaphorePermit, Semaphore},
    time::Sleep,
};

const CONNECTION_LIMIT: usize = 32;
const CONNECTION_LIFETIME: Duration = Duration::from_secs(10);

pub struct BoundedListener {
    listener: TcpListener,
    slots: Arc<Semaphore>,
    lifetime: Duration,
}

impl BoundedListener {
    pub fn new(listener: TcpListener) -> Self {
        Self {
            listener,
            slots: Arc::new(Semaphore::new(CONNECTION_LIMIT)),
            lifetime: CONNECTION_LIFETIME,
        }
    }
}

impl Listener for BoundedListener {
    type Io = BoundedStream;
    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            match self.listener.accept().await {
                Ok((stream, address)) => {
                    if let Ok(permit) = self.slots.clone().try_acquire_owned() {
                        return (
                            BoundedStream {
                                stream,
                                _permit: permit,
                                deadline: Box::pin(tokio::time::sleep(self.lifetime)),
                            },
                            address,
                        );
                    }
                    // Reject before Axum creates a connection task. No wait queue.
                    drop(stream);
                }
                Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
            }
        }
    }

    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }
}

pub struct BoundedStream {
    stream: TcpStream,
    _permit: OwnedSemaphorePermit,
    deadline: Pin<Box<Sleep>>,
}

impl BoundedStream {
    fn check_deadline(&mut self, cx: &mut Context<'_>) -> io::Result<()> {
        if self.deadline.as_mut().poll(cx).is_ready() {
            Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "local HTTP connection deadline",
            ))
        } else {
            Ok(())
        }
    }
}

impl AsyncRead for BoundedStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if let Err(error) = this.check_deadline(cx) {
            return Poll::Ready(Err(error));
        }
        Pin::new(&mut this.stream).poll_read(cx, buffer)
    }
}

impl AsyncWrite for BoundedStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        if let Err(error) = this.check_deadline(cx) {
            return Poll::Ready(Err(error));
        }
        Pin::new(&mut this.stream).poll_write(cx, bytes)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if let Err(error) = this.check_deadline(cx) {
            return Poll::Ready(Err(error));
        }
        Pin::new(&mut this.stream).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_shutdown(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn fixture(
        limit: usize,
        lifetime: Duration,
    ) -> (SocketAddr, Arc<Semaphore>, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let slots = Arc::new(Semaphore::new(limit));
        let bounded = BoundedListener {
            listener,
            slots: slots.clone(),
            lifetime,
        };
        let task = tokio::spawn(async {
            let router = axum::Router::new().route("/", axum::routing::get(|| async { "ok" }));
            axum::serve(bounded, router).await.unwrap();
        });
        (address, slots, task)
    }

    async fn wait_slots(slots: &Semaphore, count: usize) {
        tokio::time::timeout(Duration::from_secs(2), async {
            while slots.available_permits() != count {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap();
    }

    async fn closed(peer: &mut TcpStream) {
        let result = tokio::time::timeout(Duration::from_secs(2), peer.read(&mut [0; 1]))
            .await
            .unwrap();
        assert!(matches!(result, Ok(0) | Err(_)));
    }

    #[tokio::test]
    async fn incomplete_headers_are_admitted_bounded_rejected_and_released() {
        let (address, slots, task) = fixture(2, Duration::from_millis(300)).await;
        let mut first = TcpStream::connect(address).await.unwrap();
        first.write_all(b"GET / HTTP/1.1\r\nHost: ").await.unwrap();
        let mut second = TcpStream::connect(address).await.unwrap();
        second.write_all(b"GET / HTTP/1.1\r\nHost: ").await.unwrap();
        wait_slots(&slots, 0).await;
        let mut refused = TcpStream::connect(address).await.unwrap();
        closed(&mut refused).await;
        closed(&mut first).await;
        closed(&mut second).await;
        wait_slots(&slots, 2).await;
        let mut recovered = TcpStream::connect(address).await.unwrap();
        recovered
            .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .await
            .unwrap();
        let mut response = Vec::new();
        tokio::time::timeout(Duration::from_secs(2), recovered.read_to_end(&mut response))
            .await
            .unwrap()
            .unwrap();
        assert!(response.starts_with(b"HTTP/1.1 200"));
        task.abort();
        let _ = task.await;
    }

    #[tokio::test]
    async fn slow_header_bytes_do_not_extend_absolute_connection_deadline() {
        let (address, slots, task) = fixture(1, Duration::from_millis(250)).await;
        let mut peer = TcpStream::connect(address).await.unwrap();
        wait_slots(&slots, 0).await;
        for _ in 0..3 {
            peer.write_all(b"G").await.unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        closed(&mut peer).await;
        wait_slots(&slots, 1).await;
        task.abort();
        let _ = task.await;
    }
}
