//! Link operation.
use crate::SolverError;
use super::{TreeSolverError, TreeSolver, counted::Counted, unique::TryUnique};

use ast::Span;
use futures::{Stream, TryStreamExt, channel::mpsc::UnboundedSender};
use intorinf::IntOrInf;
use provider::{
    PageInfo, DataProvider,
    LinksConfig, BackLinksConfig, EmbedsConfig, PrefixConfig,
};

macro_rules! simple_query {
    ($vis:vis, $name:ident, $config:ident, $trait_method:ident, $type:ident) => {
        $vis type $type<St, P>
        where
            P: DataProvider + Clone,
            St: Stream<Item=Result<PageInfo, SolverError<TreeSolver<P>>>>,
        = impl Stream<Item=Result<PageInfo, SolverError<TreeSolver<P>>>>;

        $vis fn $name<St, P>(
            stream: St,
            provider: P,
            config: $config,
            span: Span,
            limit: IntOrInf,
            warning_sender: UnboundedSender<SolverError<TreeSolver<P>>>,
        ) -> $type<St, P>
        where
            P: DataProvider + Clone,
            St: Stream<Item=Result<PageInfo, SolverError<TreeSolver<P>>>>,
        {
            let span_2 = span.clone();
            let stream = stream.map_ok(move |x| {
                let span = span_2.clone();
                provider.$trait_method(x.get_title().unwrap(), &config)
                .map_err(move |e| {
                    let span = span.clone();
                    SolverError::from_solver_error(span.clone(), TreeSolverError::Provider(e))
                })
            }).try_flatten();
            TryUnique::new(Counted::new(stream, span, limit, warning_sender))
        }
    }
}

simple_query!(pub(crate), make_links_stream, LinksConfig, get_links, LinksStream);
simple_query!(pub(crate), make_backlinks_stream, BackLinksConfig, get_backlinks, BacklinksStream);
simple_query!(pub(crate), make_embeds_stream, EmbedsConfig, get_embeds, EmbedsStream);
simple_query!(pub(crate), make_prefix_stream, PrefixConfig, get_prefix, PrefixStream);
