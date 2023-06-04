//! Retrive information of a page.

use ast::Span;
use core::{pin::Pin, task::{Context, Poll}};
use crate::{SolverResult, RuntimeWarning, RuntimeError};
use futures::{Stream, ready, stream::FusedStream};
use provider::DataProvider;
use trio_result::TrioResult;

#[pin_project::pin_project]
pub struct PageInfoStream<P>
where
    P: DataProvider,
{
    #[pin] stream: P::PageInfoRawStream,
    span: Span,
}

impl<P> PageInfoStream<P>
where
    P: DataProvider
{
    pub fn new(titles: &[String], provider: P, span: Span,) -> Self {
        let st = provider.get_page_info_from_raw(titles.to_owned());
        Self {
            stream: st,
            span,
        }
    }
}

impl<P> FusedStream for PageInfoStream<P>
where
    P: DataProvider,
    P::PageInfoRawStream: FusedStream,
{
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated()
    }
}

impl<P> Stream for PageInfoStream<P>
where
    P: DataProvider,
{
    type Item = SolverResult<P>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        Poll::Ready({
            if let Some(item) = ready!(this.stream.as_mut().poll_next(cx)) {
                match item {
                    TrioResult::Ok(item) => Some(TrioResult::Ok(item)),
                    TrioResult::Warn(w) => Some(TrioResult::Warn(RuntimeWarning::Provider { span: *this.span, warn: w })),
                    TrioResult::Err(e) => Some(TrioResult::Err(RuntimeError::Provider { span: *this.span, error: e })),
                }
            } else {
                None
            }
        })
    }
}
