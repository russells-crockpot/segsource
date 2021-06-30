use futures_core::ready;
use pin_project_lite::pin_project;
use std::{
    future::Future,
    io,
    marker::Unpin,
    mem,
    path::Path,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    fs,
    io::{AsyncBufRead, BufReader},
};
pin_project! {
    pub struct VecFromAsyncBufread<R: AsyncBufRead> {
        #[pin]
        reader: R,
        buf: Vec<u8>,
    }
}

impl<R: AsyncBufRead + Unpin> Future for VecFromAsyncBufread<R> {
    type Output = io::Result<Vec<u8>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut me = self.project();
        let data = {
            match ready!(me.reader.as_mut().poll_fill_buf(cx)) {
                Err(error) => return Poll::Ready(Err(error)),
                Ok(d) => d,
            }
        };
        if data.is_empty() {
            Poll::Ready(Ok(mem::take(me.buf)))
        } else {
            me.buf.extend_from_slice(&data);
            let len = data.len();
            me.reader.as_mut().consume(len);
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

pub fn u8_vec_from_async_bufread<R: AsyncBufRead + Unpin>(
    reader: R,
    capacity: Option<usize>,
) -> VecFromAsyncBufread<R> {
    let buf = if let Some(size) = capacity {
        Vec::with_capacity(size)
    } else {
        Vec::new()
    };
    VecFromAsyncBufread { reader, buf }
}

pub async fn async_u8_vec_from_file<P>(path: P) -> io::Result<Vec<u8>>
where
    P: AsRef<Path> + Sync + Send,
{
    let md = fs::metadata(&path).await?;
    let file = fs::File::open(path).await?;
    let reader = BufReader::new(file);
    u8_vec_from_async_bufread(reader, Some(md.len() as usize)).await
}

#[cfg(feature = "bytes")]
mod with_bytes {
    use bytes::{Bytes, BytesMut};
    use futures_core::ready;
    use pin_project_lite::pin_project;
    use std::{
        future::Future,
        io,
        marker::Unpin,
        mem,
        path::Path,
        pin::Pin,
        task::{Context, Poll},
    };
    use tokio::{
        fs,
        io::{AsyncBufRead, BufReader},
    };

    pin_project! {
        pub struct BytesFromAsyncBufread<R: AsyncBufRead> {
            #[pin]
            reader: R,
            buf: BytesMut,
        }
    }

    impl<R: AsyncBufRead + Unpin> Future for BytesFromAsyncBufread<R> {
        type Output = io::Result<Bytes>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
            let mut me = self.project();
            let data = {
                match ready!(me.reader.as_mut().poll_fill_buf(cx)) {
                    Err(error) => return Poll::Ready(Err(error)),
                    Ok(d) => d,
                }
            };
            if data.is_empty() {
                Poll::Ready(Ok(Bytes::from(mem::take(me.buf))))
            } else {
                me.buf.extend_from_slice(&data);
                let len = data.len();
                me.reader.as_mut().consume(len);
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }

    pub fn bytes_from_async_bufread<R: AsyncBufRead + Unpin>(
        reader: R,
        capacity: Option<usize>,
    ) -> BytesFromAsyncBufread<R> {
        let buf = if let Some(size) = capacity {
            BytesMut::with_capacity(size)
        } else {
            BytesMut::new()
        };
        BytesFromAsyncBufread { reader, buf }
    }

    pub async fn async_bytes_from_file<P>(path: P) -> io::Result<Bytes>
    where
        P: AsRef<Path> + Sync + Send,
    {
        let md = fs::metadata(&path).await?;
        let file = fs::File::open(path).await?;
        let reader = BufReader::new(file);
        bytes_from_async_bufread(reader, Some(md.len() as usize)).await
    }
}
#[cfg(feature = "bytes")]
pub use with_bytes::*;
