//! A recursive-tree solver

#![cfg(feature = "recursive-tree")]
#![allow(unused_variables)]

use std::collections::BTreeSet;

use interface::types::ast::{Node, Expr, Modifier, NumberOrInf};
use crate::{Answer, Solver, Error};
use provider::{PagePair, DataProvider};

pub struct RecursiveTreeSolver<'solver, D>
where
    D: DataProvider,
{
    provider: &'solver D,
    default_limit: NumberOrInf<usize>,
}

#[async_trait::async_trait]
impl<'solver, 'query, D> Solver<'solver, 'query, D> for RecursiveTreeSolver<'solver, D>
where
    D: DataProvider,
{
    type InnerError = RecursiveTreeSolverError<D>;

    async fn solve(&'solver self, ast: &'query Node) -> Result<Answer<'query, Self::InnerError>, Error<'query, Self::InnerError>> {
        let (set, warnings) = self.solve_internal(ast).await?;
        Ok(Answer {
            titles: BTreeSet::from_iter(set.into_iter().map(|(i, _)| i.title)),
            warnings,
            // _phantom: PhantomData,
        })
    }

}

impl<'solver, 'query, D> RecursiveTreeSolver<'solver, D>
where
    D: DataProvider,
{

    pub fn new(provider: &'solver D, default_limit: NumberOrInf<usize>) -> Self {
        Self { provider, default_limit }
    }

    fn convert_modifier(&'solver self, modifier: &'query Modifier) -> Modifier {
        Modifier {
            result_limit: Some(modifier.result_limit.unwrap_or(self.default_limit)),
            ..modifier.to_owned()
        }
    }

    #[async_recursion::async_recursion]
    async fn solve_internal(&'solver self, node: &'query Node) -> Result<(BTreeSet<PagePair>, Vec<Error<'query, RecursiveTreeSolverError<D>>>), Error<'query, RecursiveTreeSolverError<D>>> {
        match &node.expr {
            Expr::Page { titles } => self.handle_page(node, titles).await,
            Expr::Intersection { set1, set2 } => self.handle_intersection(node, set1, set2).await,
            Expr::Union { set1, set2 } => self.handle_union(node, set1, set2).await,
            Expr::Difference { set1, set2 } => self.handle_difference(node, set1, set2).await,
            Expr::Xor { set1, set2 } => self.handle_xor(node, set1, set2).await,
            Expr::Link { target, modifier } => self.handle_link(node, target, modifier).await,
            Expr::BackLink { target, modifier } => self.handle_backlink(node, target, modifier).await,
            Expr::Embed { target, modifier } => self.handle_embed(node, target, modifier).await,
            Expr::InCategory { target, modifier } => self.handle_incategory(node, target, modifier).await,
            Expr::Prefix { target, modifier } => self.handle_prefix(node, target, modifier).await,
            Expr::Toggle { target } => self.handle_toggle(node, target).await,
        }
    }

    async fn handle_page(&'solver self, node: &'query Node, titles: &'query BTreeSet<String>) -> Result<(BTreeSet<PagePair>, Vec<Error<'query, RecursiveTreeSolverError<D>>>), Error<'query, RecursiveTreeSolverError<D>>> {
        let result = self.provider.get_page_info(titles).await;
        match result {
            Ok((infos, warnings)) => {
                let infos = BTreeSet::from_iter(infos.into_iter());
                let warnings = warnings.into_iter().map(|w| Error { node, content: RecursiveTreeSolverError::Provider(w) }).collect();
                Ok((infos, warnings))
            },
            Err(err) => Err(Error { node, content: RecursiveTreeSolverError::Provider(err) }),
        }
    }

    async fn handle_link(&'solver self, node: &'query Node, target: &'query Node, modifier: &'query Modifier) -> Result<(BTreeSet<PagePair>, Vec<Error<'query, RecursiveTreeSolverError<D>>>), Error<'query, RecursiveTreeSolverError<D>>> {
        let (titles, mut warnings) = self.solve_internal(target).await?;
        let titles = BTreeSet::from_iter(titles.into_iter().map(|(i, _)| i.title));
        let result = self.provider.get_links(&titles, &self.convert_modifier(modifier)).await;
        match result {
            Ok((infos, new_warnings)) => {
                let infos = BTreeSet::from_iter(infos.into_iter());
                warnings.extend(new_warnings.into_iter().map(|w| Error { node, content: RecursiveTreeSolverError::Provider(w) }));
                Ok((infos, warnings))
            },
            Err(err) => Err(Error { node, content: RecursiveTreeSolverError::Provider(err) }),
        }
    }

    async fn handle_backlink(&'solver self, node: &'query Node, target: &'query Node, modifier: &'query Modifier) -> Result<(BTreeSet<PagePair>, Vec<Error<'query, RecursiveTreeSolverError<D>>>), Error<'query, RecursiveTreeSolverError<D>>> {
        let (titles, mut warnings) = self.solve_internal(target).await?;
        let titles = BTreeSet::from_iter(titles.into_iter().map(|(i, _)| i.title));
        let result = self.provider.get_backlinks(&titles, &self.convert_modifier(modifier)).await;
        match result {
            Ok((infos, new_warnings)) => {
                let infos = BTreeSet::from_iter(infos.into_iter());
                warnings.extend(new_warnings.into_iter().map(|w| Error { node, content: RecursiveTreeSolverError::Provider(w) }));
                Ok((infos, warnings))
            },
            Err(err) => Err(Error { node, content: RecursiveTreeSolverError::Provider(err) }),
        }
    }

    async fn handle_embed(&'solver self, node: &'query Node, target: &'query Node, modifier: &'query Modifier) -> Result<(BTreeSet<PagePair>, Vec<Error<'query, RecursiveTreeSolverError<D>>>), Error<'query, RecursiveTreeSolverError<D>>> {
        let (titles, mut warnings) = self.solve_internal(target).await?;
        let titles = BTreeSet::from_iter(titles.into_iter().map(|(i, _)| i.title));
        let result = self.provider.get_embeds(&titles, &self.convert_modifier(modifier)).await;
        match result {
            Ok((infos, new_warnings)) => {
                let infos = BTreeSet::from_iter(infos.into_iter());
                warnings.extend(new_warnings.into_iter().map(|w| Error { node, content: RecursiveTreeSolverError::Provider(w) }));
                Ok((infos, warnings))
            },
            Err(err) => Err(Error { node, content: RecursiveTreeSolverError::Provider(err) }),
        }
    }

    async fn handle_incategory(&'solver self, node: &'query Node, target: &'query Node, modifier: &Modifier) -> Result<(BTreeSet<PagePair>, Vec<Error<'query, RecursiveTreeSolverError<D>>>), Error<'query, RecursiveTreeSolverError<D>>> {
        let (titles, mut warnings) = self.solve_internal(target).await?;
        let titles = BTreeSet::from_iter(titles.into_iter().map(|(i, _)| i.title));
        let result = self.provider.get_category_members(&titles, &self.convert_modifier(modifier)).await;
        match result {
            Ok((infos, new_warnings)) => {
                let infos = BTreeSet::from_iter(infos.into_iter());
                warnings.extend(new_warnings.into_iter().map(|w| Error { node, content: RecursiveTreeSolverError::Provider(w) }));
                Ok((infos, warnings))
            },
            Err(err) => Err(Error { node, content: RecursiveTreeSolverError::Provider(err) }),
        }
    }

    async fn handle_prefix(&'solver self, node: &'query Node, target: &'query Node, modifier: &Modifier) -> Result<(BTreeSet<PagePair>, Vec<Error<'query, RecursiveTreeSolverError<D>>>), Error<'query, RecursiveTreeSolverError<D>>> {
        let (titles, mut warnings) = self.solve_internal(target).await?;
        let titles = BTreeSet::from_iter(titles.into_iter().map(|(i, _)| i.title));
        let result = self.provider.get_prefix(&titles, &self.convert_modifier(modifier)).await;
        match result {
            Ok((infos, new_warnings)) => {
                let infos = BTreeSet::from_iter(infos.into_iter());
                warnings.extend(new_warnings.into_iter().map(|w| Error { node, content: RecursiveTreeSolverError::Provider(w) }));
                Ok((infos, warnings))
            },
            Err(err) => Err(Error { node, content: RecursiveTreeSolverError::Provider(err) }),
        }
    }

    async fn handle_toggle(&'solver self, node: &'query Node, target: &'query Node) -> Result<(BTreeSet<PagePair>, Vec<Error<'query, RecursiveTreeSolverError<D>>>), Error<'query, RecursiveTreeSolverError<D>>> {
        let (mut set, warnings) = self.solve_internal(target).await?;
        // No page's associated page lies in virtual-namespaces.
        // If so, we assert that the associated page should not exist at all (`Bad Title`, eg. no `Topic talk` namespace).
        set.retain(|(_, asso)| asso.title.namespace() >= 0);
        let toggled_set = BTreeSet::from_iter(set.into_iter().map(|(subj, asso)| (asso, subj)));
        Ok((toggled_set, warnings))
    }

    async fn handle_intersection(&'solver self, node: &'query Node, set1: &'query Node, set2: &'query Node) -> Result<(BTreeSet<PagePair>, Vec<Error<'query, RecursiveTreeSolverError<D>>>), Error<'query, RecursiveTreeSolverError<D>>> {
        let (set1_result, set1_warnings) = self.solve_internal(set1).await?;
        let (set2_result, set2_warnings) = self.solve_internal(set2).await?;
        let intersection = set1_result.intersection(&set2_result);
        let set = BTreeSet::from_iter(intersection.cloned().collect::<BTreeSet<PagePair>>());
        let mut warnings = Vec::new();
        warnings.extend(set1_warnings);
        warnings.extend(set2_warnings);
        Ok((set, warnings))
    }

    async fn handle_xor(&'solver self, node: &'query Node, set1: &'query Node, set2: &'query Node) -> Result<(BTreeSet<PagePair>, Vec<Error<'query, RecursiveTreeSolverError<D>>>), Error<'query, RecursiveTreeSolverError<D>>> {
        let (set1_result, set1_warnings) = self.solve_internal(set1).await?;
        let (set2_result, set2_warnings) = self.solve_internal(set2).await?;
        let xor = set1_result.symmetric_difference(&set2_result);
        let set = BTreeSet::from_iter(xor.cloned().collect::<BTreeSet<PagePair>>());
        let mut warnings = Vec::new();
        warnings.extend(set1_warnings);
        warnings.extend(set2_warnings);
        Ok((set, warnings))
    }

    async fn handle_difference(&'solver self, node: &'query Node, set1: &'query Node, set2: &'query Node) -> Result<(BTreeSet<PagePair>, Vec<Error<'query, RecursiveTreeSolverError<D>>>), Error<'query, RecursiveTreeSolverError<D>>> {
        let (set1_result, set1_warnings) = self.solve_internal(set1).await?;
        let (set2_result, set2_warnings) = self.solve_internal(set2).await?;
        let diff = set1_result.difference(&set2_result);
        let set = BTreeSet::from_iter(diff.cloned().collect::<BTreeSet<PagePair>>());
        let mut warnings = Vec::new();
        warnings.extend(set1_warnings);
        warnings.extend(set2_warnings);
        Ok((set, warnings))
    }

    async fn handle_union(&'solver self, node: &'query Node, set1: &'query Node, set2: &'query Node) -> Result<(BTreeSet<PagePair>, Vec<Error<'query, RecursiveTreeSolverError<D>>>), Error<'query, RecursiveTreeSolverError<D>>> {
        let (set1_result, set1_warnings) = self.solve_internal(set1).await?;
        let (set2_result, set2_warnings) = self.solve_internal(set2).await?;
        let union = set1_result.union(&set2_result);
        let set = BTreeSet::from_iter(union.cloned().collect::<BTreeSet<PagePair>>());
        let mut warnings = Vec::new();
        warnings.extend(set1_warnings);
        warnings.extend(set2_warnings);
        Ok((set, warnings))
    }

}

#[derive(Debug)]
pub enum RecursiveTreeSolverError<P>
where
    P: DataProvider,
{
    Provider(P::Error),
}
/*
impl<P: DataProvider> From<P::Error> for RecursiveTreeSolverError<P> {
    fn from(e: P::Error) -> Self {
        Self::Provider(e)
    }
}
*/
impl<P> core::fmt::Display for RecursiveTreeSolverError<P>
where
    P: DataProvider,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Provider(e) => write!(f, "data provider error: {}", e),
        }
    }
}

impl<P> std::error::Error for RecursiveTreeSolverError<P> where P: DataProvider {}
