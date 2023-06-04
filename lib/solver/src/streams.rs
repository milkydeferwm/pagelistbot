use core::{pin::Pin, task::{Context, Poll}};
use crate::SolverResult;
use futures::Stream;
use provider::DataProvider;

mod category;
mod counted;
mod cut;
mod pageinfo;
mod query;
mod set;
mod toggle;
mod unique;

pub(crate) use category::CategoryMembersStream;
pub(crate) use counted::Counted;
pub(crate) use cut::CutError;
pub(crate) use pageinfo::PageInfoStream;
pub(crate) use query::{
    LinksStream,
    BacklinksStream,
    EmbedsStream,
    PrefixStream,
};
pub(crate) use set::{
    IntersectionStream,
    UnionStream,
    DifferenceStream,
    XorStream,
};
pub(crate) use toggle::ToggleStream;
pub(crate) use unique::Unique;

type PinnedSolverStream<P> = Pin<Box<SolverStream<P>>>;

#[pin_project::pin_project(project = SolverStreamProj)]
#[non_exhaustive]
pub enum SolverStream<P>
where
    P: DataProvider + Clone,
{
    /// Stream by pageinfo
    PageInfo(#[pin] PageInfoStream<P>),

    /// Stream by intersection operation
    Intersection(#[pin] IntersectionStream<PinnedSolverStream<P>, PinnedSolverStream<P>, P>),
    /// Stream by union operation
    Union(#[pin] UnionStream<PinnedSolverStream<P>, PinnedSolverStream<P>, P>),
    /// Stream by difference operation
    Difference(#[pin] DifferenceStream<PinnedSolverStream<P>, PinnedSolverStream<P>, P>),
    /// Stream by xor operation
    Xor(#[pin] XorStream<PinnedSolverStream<P>, PinnedSolverStream<P>, P>),

    /// Stream by links operation
    Links(#[pin] LinksStream<PinnedSolverStream<P>, P>),
    /// Stream by backlinks operation
    Backlinks(#[pin] BacklinksStream<PinnedSolverStream<P>, P>),
    /// Stream by embeds operation
    Embeds(#[pin] EmbedsStream<PinnedSolverStream<P>, P>),
    /// Stream by perfix operation
    Prefix(#[pin] PrefixStream<PinnedSolverStream<P>, P>),
    /// Stream by incat operation
    CategoryMembers(#[pin] CategoryMembersStream<PinnedSolverStream<P>, P>),
    /// Stream by toggle operation
    Toggle(#[pin] ToggleStream<PinnedSolverStream<P>, P>),

    /// Stream that tracks number of elements yielded.
    Counted(#[pin] Counted<PinnedSolverStream<P>, P>),
    /// Stream that fuses once an error is yielded.
    CutError(#[pin] CutError<PinnedSolverStream<P>, P>),
    /// Stream that removes any duplicate item.
    Unique(#[pin] Unique<PinnedSolverStream<P>, P>),
}

impl<P> Stream for SolverStream<P>
where
    P: DataProvider + Clone,
{
    type Item = SolverResult<P>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this {
            SolverStreamProj::PageInfo(s) => s.poll_next(cx),
            SolverStreamProj::Intersection(s) => s.poll_next(cx),
            SolverStreamProj::Union(s) => s.poll_next(cx),
            SolverStreamProj::Difference(s) => s.poll_next(cx),
            SolverStreamProj::Xor(s) => s.poll_next(cx),
            SolverStreamProj::Links(s) => s.poll_next(cx),
            SolverStreamProj::Backlinks(s) => s.poll_next(cx),
            SolverStreamProj::Embeds(s) => s.poll_next(cx),
            SolverStreamProj::Prefix(s) => s.poll_next(cx),
            SolverStreamProj::CategoryMembers(s) => s.poll_next(cx),
            SolverStreamProj::Toggle(s) => s.poll_next(cx),
            SolverStreamProj::Counted(s) => s.poll_next(cx),
            SolverStreamProj::CutError(s) => s.poll_next(cx),
            SolverStreamProj::Unique(s) => s.poll_next(cx),
        }
    }
}
