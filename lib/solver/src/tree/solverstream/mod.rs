//! A collection of stream types. Stream are structured like a tree to provide solving.

use core::{pin::Pin, task::{Context, Poll}};
use crate::{SolverError, cvt_attr};
use ast::Expression;
use futures::{Stream, channel::mpsc::UnboundedSender};
use intorinf::IntOrInf;
use super::{TreeSolverError, TreeSolver};
use provider::{DataProvider, PageInfo};

mod categorymembers;
mod setop;
mod pageinfo;
mod simplequery;
mod toggle;
mod unique;
mod counted;

type PinnedUniversalStream<P> = Pin<Box<UniversalStream<P>>>;

#[pin_project::pin_project(project = UniversalStreamProj)]
#[non_exhaustive]
pub(crate) enum UniversalStream<P>
where
    P: DataProvider + Clone,
{
    /// Stream by pageinfo
    PageInfo(#[pin] pageinfo::PageInfoStream<P>),
    // set operations
    /// Stream by intersection operation
    Intersection(#[pin] setop::IntersectionStream<PinnedUniversalStream<P>, PinnedUniversalStream<P>, P>),
    /// Stream by union operation
    Union(#[pin] setop::UnionStream<PinnedUniversalStream<P>, PinnedUniversalStream<P>, P>),
    /// Stream by difference operation
    Difference(#[pin] setop::DifferenceStream<PinnedUniversalStream<P>, PinnedUniversalStream<P>, P>),
    /// Stream by xor operation
    Xor(#[pin] setop::XorStream<PinnedUniversalStream<P>, PinnedUniversalStream<P>, P>),
    // unary operations
    /// Stream by links operation
    Links(#[pin]simplequery::LinksStream<PinnedUniversalStream<P>, P>),
    /// Stream by backlinks operation
    Backlinks(#[pin] simplequery::BacklinksStream<PinnedUniversalStream<P>, P>),
    /// Stream by embeds operation
    Embeds(#[pin] simplequery::EmbedsStream<PinnedUniversalStream<P>, P>),
    /// Stream by perfix operation
    Prefix(#[pin] simplequery::PrefixStream<PinnedUniversalStream<P>, P>),
    /// Stream by incat operation
    CategoryMembers(#[pin] categorymembers::CategoryMembersStream<PinnedUniversalStream<P>, P>),
    /// Stream by toggle operation
    Toggle(#[pin] toggle::ToggleStream<PinnedUniversalStream<P>, P>),
}

impl<P> Stream for UniversalStream<P>
where
    P: DataProvider + Clone,
{
    type Item = Result<PageInfo, SolverError<TreeSolver<P>>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this {
            UniversalStreamProj::PageInfo(s) => s.poll_next(cx),
            UniversalStreamProj::Intersection(s) => s.poll_next(cx),
            UniversalStreamProj::Union(s) => s.poll_next(cx),
            UniversalStreamProj::Difference(s) => s.poll_next(cx),
            UniversalStreamProj::Xor(s) => s.poll_next(cx),
            UniversalStreamProj::Links(s) => s.poll_next(cx),
            UniversalStreamProj::Backlinks(s) => s.poll_next(cx),
            UniversalStreamProj::Embeds(s) => s.poll_next(cx),
            UniversalStreamProj::Prefix(s) => s.poll_next(cx),
            UniversalStreamProj::CategoryMembers(s) => s.poll_next(cx),
            UniversalStreamProj::Toggle(s) => s.poll_next(cx),
        }
    }
}

impl<P> UniversalStream<P>
where
    P: DataProvider + Clone,
{
    pub(crate) fn from_expr(expr: &Expression, provider: P, default_limit: IntOrInf, warning_sender: UnboundedSender<SolverError<TreeSolver<P>>>) -> Result<Self, SolverError<TreeSolver<P>>> {
        match expr {
            Expression::Add(inner) => {
                let st1 = {
                    let warning_sender = warning_sender.clone();
                    Self::from_expr(&inner.expr1, provider.clone(), default_limit, warning_sender)?
                };
                let st2 = Self::from_expr(&inner.expr2, provider, default_limit, warning_sender)?;
                let st = setop::UnionStream::new(Box::pin(st1), Box::pin(st2));
                Ok(Self::Union(st))
            },
            Expression::And(inner) => {
                let st1 = {
                    let warning_sender = warning_sender.clone();
                    Self::from_expr(&inner.expr1, provider.clone(), default_limit, warning_sender)?
                };
                let st2 = Self::from_expr(&inner.expr2, provider, default_limit, warning_sender)?;
                let st = setop::IntersectionStream::new(Box::pin(st1), Box::pin(st2));
                Ok(Self::Intersection(st))
            },
            Expression::Sub(inner) => {
                let st1 = {
                    let warning_sender = warning_sender.clone();
                    Self::from_expr(&inner.expr1, provider.clone(), default_limit, warning_sender)?
                };
                let st2 = Self::from_expr(&inner.expr2, provider, default_limit, warning_sender)?;
                let st = setop::DifferenceStream::new(Box::pin(st1), Box::pin(st2));
                Ok(Self::Difference(st))
            },
            Expression::Xor(inner) => {
                let st1 = {
                    let warning_sender = warning_sender.clone();
                    Self::from_expr(&inner.expr1, provider.clone(), default_limit, warning_sender)?
                };
                let st2 = Self::from_expr(&inner.expr2, provider, default_limit, warning_sender)?;
                let st = setop::XorStream::new(Box::pin(st1), Box::pin(st2));
                Ok(Self::Xor(st))
            },
            Expression::Paren(inner) => Self::from_expr(&inner.expr, provider, default_limit, warning_sender),
            Expression::Page(inner) => {
                let pages = inner.vals.iter().map(|lit| lit.val.to_owned()).collect::<Vec<_>>();
                let st = pageinfo::make_pageinfo_stream(pages, provider, expr.get_span());
                Ok(Self::PageInfo(st))
            },
            Expression::Link(inner) => {
                let (config, limit) = cvt_attr::links_config_from_attributes(&inner.attributes)?;
                let st0 = {
                    let warning_sender = warning_sender.clone();
                    Self::from_expr(&inner.expr, provider.clone(), default_limit, warning_sender)?
                };
                let st = simplequery::make_links_stream(
                    Box::pin(st0),
                    provider,
                    config,
                    expr.get_span(),
                    limit.unwrap_or(default_limit),
                    warning_sender,
                );
                Ok(Self::Links(st))
            },
            Expression::LinkTo(inner) => {
                let (config, limit) = cvt_attr::backlinks_config_from_attributes(&inner.attributes)?;
                let st0 = {
                    let warning_sender = warning_sender.clone();
                    Self::from_expr(&inner.expr, provider.clone(), default_limit, warning_sender)?
                };
                let st = simplequery::make_backlinks_stream(
                    Box::pin(st0),
                    provider,
                    config,
                    expr.get_span(),
                    limit.unwrap_or(default_limit),
                    warning_sender,
                );
                Ok(Self::Backlinks(st))
            },
            Expression::Embed(inner) => {
                let (config, limit) = cvt_attr::embeds_config_from_attributes(&inner.attributes)?;
                let st0 = {
                    let warning_sender = warning_sender.clone();
                    Self::from_expr(&inner.expr, provider.clone(), default_limit, warning_sender)?
                };
                let st = simplequery::make_embeds_stream(
                    Box::pin(st0),
                    provider,
                    config,
                    expr.get_span(),
                    limit.unwrap_or(default_limit),
                    warning_sender,
                );
                Ok(Self::Embeds(st))
            },
            Expression::InCat(inner) => {
                let (config, limit, depth) = cvt_attr::categorymembers_config_from_attributes(&inner.attributes)?;
                let st0 = {
                    let warning_sender = warning_sender.clone();
                    Self::from_expr(&inner.expr, provider.clone(), default_limit, warning_sender)?
                };
                let st = categorymembers::make_categorymembers_stream(
                    Box::pin(st0),
                    provider,
                    config,
                    expr.get_span(),
                    limit.unwrap_or(default_limit),
                    depth.unwrap_or(IntOrInf::Int(0)),
                    warning_sender,
                );
                Ok(Self::CategoryMembers(st))
            },
            Expression::Prefix(inner) => {
                let (config, limit) = cvt_attr::prefix_config_from_attributes(&inner.attributes)?;
                let st0 = {
                    let warning_sender = warning_sender.clone();
                    Self::from_expr(&inner.expr, provider.clone(), default_limit, warning_sender)?
                };
                let st = simplequery::make_prefix_stream(
                    Box::pin(st0),
                    provider,
                    config,
                    expr.get_span(),
                    limit.unwrap_or(default_limit),
                    warning_sender,
                );
                Ok(Self::Prefix(st))
            },
            Expression::Toggle(inner) => {
                let st0 = {
                    Self::from_expr(&inner.expr, provider, default_limit, warning_sender)?
                };
                let st = toggle::make_toggle_stream(Box::pin(st0));
                Ok(Self::Toggle(st))
            },
            _ => unimplemented!(),
        }
    }
}
