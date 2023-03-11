//! Link operation.
use core::{pin::Pin, task::{Context, Poll}};
use crate::SolverError;
use super::{TreeSolverError, TreeSolver};

use ast::Span;
use futures::{stream::{TryStream, Empty, self}, Stream, channel::mpsc::UnboundedSender, future::Either};
use intorinf::IntOrInf;
use mwtitle::Title;
use provider::{
    Pair, PageInfo, DataProvider,
    LinksConfig, BackLinksConfig, EmbedsConfig, PrefixConfig,
};
use std::collections::BTreeSet;

macro_rules! simple_query {
    ($vis:vis, $name:ident, $trait_stream:ident, $trait_method:ident, $config:ident) => {
        #[pin_project::pin_project]
        #[must_use = "streams do nothing unless you poll them"]
        $vis struct $name<'e, F, P>
        where
            F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<'e, TreeSolver<P>>>,
            P: DataProvider + Clone,
        {
            #[pin] source: F,
            #[pin] workload: Either<Empty<<<P as DataProvider>::$trait_stream as Stream>::Item>, <P as DataProvider>::$trait_stream>,
            provider: P,
            config: $config,
            span: Span<'e>,
            result_limit: IntOrInf,
            count: IntOrInf,
            produced_pages: BTreeSet<Title>,
            warning_sender: UnboundedSender<SolverError<'e, TreeSolver<P>>>,
        }

        impl<'e, F, P> $name<'e, F, P>
        where
            F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<'e, TreeSolver<P>>>,
            P: DataProvider + Clone,
        {
            pub fn new(
                stream: F,
                provider: P,
                config: $config,
                span: Span<'e>,
                result_limit: IntOrInf,
                warning_sender: UnboundedSender<SolverError<'e, TreeSolver<P>>>
            ) -> Self
            {
                Self {
                    source: stream,
                    workload: Either::Left(stream::empty()),
                    provider,
                    config,
                    span,
                    result_limit,
                    count: IntOrInf::Int(0),
                    produced_pages: BTreeSet::new(),
                    warning_sender,
                }
            }
        }

        impl<'e, F, P> Stream for $name<'e, F, P>
        where
            F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<'e, TreeSolver<P>>>,
            P: DataProvider + Clone,
        {
            type Item = Result<Pair<PageInfo>, SolverError<'e, TreeSolver<P>>>;

            fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                let mut this = self.project();
                match *this.workload {
                    Either::Left(_) => {
                        // poll source stream.
                        let source_poll_result = this.source.as_mut().try_poll_next(cx)?;
                        match source_poll_result {
                            Poll::Pending => Poll::Pending,
                            Poll::Ready(None) => Poll::Ready(None),
                            Poll::Ready(Some(pair)) => {
                                let title = pair.0.get_title().unwrap().to_owned();
                                this.workload.set(Either::Right(this.provider.$trait_method([title], &this.config)));
                                Poll::Pending
                            },
                        }
                    },
                    Either::Right(_) => {
                        // poll data stream.
                        let data_poll_result = this.workload.as_mut().try_poll_next(cx);
                        match data_poll_result {
                            Poll::Pending => Poll::Pending,
                            Poll::Ready(None) => {
                                this.workload.set(Either::Left(stream::empty()));
                                Poll::Pending
                            },
                            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(SolverError::from_solver_error(*this.span, TreeSolverError::Provider(e))))),
                            Poll::Ready(Some(Ok(pair))) => {
                                let title = pair.0.get_title().unwrap().to_owned();    // SAFETY: infalliable.
                                *this.count += 1;
                                if *this.count >= *this.result_limit {
                                    // send warning.
                                    let warning = SolverError::from_solver_error(*this.span, TreeSolverError::ResultLimitExceeded(*this.result_limit));
                                    this.warning_sender.unbounded_send(warning).expect("cannot send warning");
                                    Poll::Ready(None)
                                } else if this.produced_pages.contains(&title) {
                                    Poll::Pending
                                } else {
                                    this.produced_pages.insert(title);
                                    Poll::Ready(Some(Ok(pair)))
                                }
                            }
                        }
                    }
                }
            }
        }

        unsafe impl<'e, F, P> Send for $name<'e, F, P>
        where
            F: TryStream<Ok=Pair<PageInfo>, Error=SolverError<'e, TreeSolver<P>>> + Send,
            P: DataProvider + Clone + Send,
            <P as DataProvider>::$trait_stream: Send,
        {}
    }
}

simple_query!(pub(crate), LinksStream, LinksStream, get_links, LinksConfig);
simple_query!(pub(crate), BacklinksStream, BacklinksStream, get_backlinks, BackLinksConfig);
simple_query!(pub(crate), EmbedsStream, EmbedsStream, get_embeds, EmbedsConfig);
simple_query!(pub(crate), PrefixStream, PrefixStream, get_prefix, PrefixConfig);

