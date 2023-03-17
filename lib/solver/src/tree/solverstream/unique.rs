//! Make the output unique.

use core::{cmp::Ord, fmt, pin::Pin, task::{Context, Poll}};
use futures::{Stream, TryStream, ready, stream::FusedStream};
use pin_project::pin_project;
use std::collections::BTreeSet;

#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub struct TryUnique<St>
where
    St: TryStream,
{
    #[pin] stream: St,
    yielded: BTreeSet<St::Ok>,
}

impl<St> TryUnique<St>
where
    St: TryStream,
{
    pub fn new(stream: St) -> Self {
        Self {
            stream,
            yielded: BTreeSet::new(),
        }
    }
}

impl<St> fmt::Debug for TryUnique<St>
where
    St: TryStream + fmt::Debug,
    St::Ok: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TryUnique")
            .field("stream", &self.stream)
            .field("yielded", &self.yielded)
            .finish()
    }
}

impl<St> FusedStream for TryUnique<St>
where
    Self: Stream,
    St: TryStream + FusedStream,
{
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated()
    }
}

impl<St> Stream for TryUnique<St>
where
    St: TryStream,
    St::Ok: Clone + Ord,
{
    type Item = Result<St::Ok, St::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        Poll::Ready(loop {
            if let Some(item) = ready!(this.stream.as_mut().try_poll_next(cx)?) {
                if !this.yielded.contains(&item) {
                    this.yielded.insert(item.clone());
                    break Some(Ok(item));
                }
            } else {
                break None;
            }
        })
    }
}
