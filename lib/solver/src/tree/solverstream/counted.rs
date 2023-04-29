//! Make the output counted.

use ast::Span;
use core::{fmt, pin::Pin, task::{Context, Poll}};
use crate::SolverError;
use futures::{Stream, TryStream, ready, stream::FusedStream, channel::mpsc::UnboundedSender};
use intorinf::IntOrInf;
use pin_project::pin_project;
use provider::DataProvider;
use super::{TreeSolver, TreeSolverError};

#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub struct Counted<St, P>
where
    St: TryStream,
    P: DataProvider + Clone,
{
    #[pin] stream: St,
    span: Span,
    count: IntOrInf,
    limit: IntOrInf,
    finish: bool,
    warning_sender: UnboundedSender<SolverError<TreeSolver<P>>>,
}

impl<St, P> Counted<St, P>
where
    St: TryStream,
    P: DataProvider + Clone,
{
    pub fn new(stream: St, span: Span, limit: IntOrInf, warning_sender: UnboundedSender<SolverError<TreeSolver<P>>>) -> Self {
        Self {
            stream,
            span,
            count: IntOrInf::Int(0),
            limit,
            finish: false,
            warning_sender,
        }
    }
}

impl<St, P> fmt::Debug for Counted<St, P>
where
    St: TryStream + fmt::Debug,
    St::Ok: fmt::Debug,
    P: DataProvider + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Counted")
            .field("stream", &self.stream)
            .field("span", &self.span)
            .field("count", &self.count)
            .field("limit", &self.limit)
            .field("finish", &self.finish)
            .field("warning_sender", &self.warning_sender)
            .finish()
    }
}

impl<St, P> FusedStream for Counted<St, P>
where
    Self: Stream,
    St: TryStream,
    P: DataProvider + Clone,
{
    fn is_terminated(&self) -> bool {
        self.finish
    }
}

impl<St, P> Stream for Counted<St, P>
where
    St: TryStream,
    P: DataProvider + Clone,
{
    type Item = Result<St::Ok, St::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        Poll::Ready({
            if let Some(item) = ready!(this.stream.as_mut().try_poll_next(cx)?) {
                if *this.finish {
                    None
                } else {
                    *this.count += 1;
                    if *this.count <= *this.limit {
                        Some(Ok(item))
                    } else {
                        *this.finish = true;
                        // send warning
                        let warning = SolverError::from_solver_error(*this.span, TreeSolverError::ResultLimitExceeded(*this.limit));
                        this.warning_sender.unbounded_send(warning).expect("cannot send warning");
                        None
                    }
                }
            } else {
                None
            }
        })
    }
}
