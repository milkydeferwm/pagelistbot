use ast::Span;
use core::{pin::Pin, task::{Context, Poll}};
use crate::{SolverResult, RuntimeWarning, RuntimeError};
use futures::{Stream, ready};
use provider::DataProvider;
use trio_result::TrioResult;

macro_rules! query {
    ($name:ident, $config:ident, $method:ident, $stream:ident) => {
        use provider::$config;

        #[pin_project::pin_project]
        #[must_use = "streams do nothing unless polled"]
        pub struct $name<St, P>
        where
            St: Stream<Item=SolverResult<P>>,
            P: DataProvider,
        {
            #[pin] source: St,
            #[pin] stream: Option<P::$stream>,
            provider: P,
            config: $config,
            span: Span,
        }

        impl<St, P> $name<St, P>
        where
            St: Stream<Item=SolverResult<P>>,
            P: DataProvider,
        {
            pub fn new(source: St, provider: P, config: $config, span: Span) -> Self {
                Self {
                    source,
                    stream: None,
                    provider,
                    config,
                    span,
                }
            }
        }

        impl<St, P> Stream for $name<St, P>
        where
            St: Stream<Item=SolverResult<P>>,
            P: DataProvider,
        {
            type Item = SolverResult<P>;

            fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                let mut this = self.project();
                Poll::Ready(loop {
                    if let Some(s) = this.stream.as_mut().as_pin_mut() {
                        // current stream is active.
                        if let Some(item) = ready!(s.poll_next(cx)) {
                            break match item {
                                TrioResult::Ok(item) => Some(TrioResult::Ok(item)),
                                TrioResult::Warn(w) => Some(TrioResult::Warn(RuntimeWarning::Provider { span: *this.span, warn: w })),
                                TrioResult::Err(e) => Some(TrioResult::Err(RuntimeError::Provider { span: *this.span, error: e })),
                            };
                        } else {
                            // current stream is exhausted.
                            this.stream.set(None);
                        }
                    } else if let Some(s) = ready!(this.source.as_mut().poll_next(cx)) {
                        // next item is available.
                        if let TrioResult::Ok(s) = s {  // getting next item successful.
                            let t = match s.get_title() {
                                Ok(t) => t,
                                Err(w) => break Some(TrioResult::Err(RuntimeError::PageInfo { span: *this.span, error: w })),
                            };
                            let next = this.provider.$method(t, &this.config);
                            this.stream.set(Some(next));
                        } else {    // getting next item failure or warning.
                            break Some(s);
                        }
                    } else {
                        break None;
                    }
                })
            }
        }
    }
}

query!(LinksStream, LinksConfig, get_links, LinksStream);
query!(BacklinksStream, BackLinksConfig, get_backlinks, BacklinksStream);
query!(EmbedsStream, EmbedsConfig, get_embeds, EmbedsStream);
query!(PrefixStream, PrefixConfig, get_prefix, PrefixStream);
