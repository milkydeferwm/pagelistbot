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
        $vis type $type<'e, St, P>
        where
            P: DataProvider + Clone,
            St: Stream<Item=Result<PageInfo, SolverError<'e, TreeSolver<P>>>>,
        = impl Stream<Item=Result<PageInfo, SolverError<'e, TreeSolver<P>>>>;

        $vis fn $name<'e, St, P>(
            stream: St,
            provider: P,
            config: $config,
            span: Span<'e>,
            limit: IntOrInf,
            warning_sender: UnboundedSender<SolverError<'e, TreeSolver<P>>>,
        ) -> $type<'e, St, P>
        where
            P: DataProvider + Clone,
            St: Stream<Item=Result<PageInfo, SolverError<'e, TreeSolver<P>>>>,
        {
            let stream = stream.map_ok(
                move |x| provider.$trait_method(x.get_title().unwrap(), &config)
                    .map_err(move |e| SolverError::from_solver_error(span, TreeSolverError::Provider(e)))
            ).try_flatten();
            TryUnique::new(Counted::new(stream, span, limit, warning_sender))
        }
    }
}

simple_query!(pub(crate), make_links_stream, LinksConfig, get_links, LinksStream);
simple_query!(pub(crate), make_backlinks_stream, BackLinksConfig, get_backlinks, BacklinksStream);
simple_query!(pub(crate), make_embeds_stream, EmbedsConfig, get_embeds, EmbedsStream);
simple_query!(pub(crate), make_prefix_stream, PrefixConfig, get_prefix, PrefixStream);
