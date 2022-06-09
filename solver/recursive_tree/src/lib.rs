//! A recursive-tree solver

#![allow(unused_variables)]

use std::collections::BTreeSet;

use mwtitle::Title;
use pagelistbot_parser::ast::{Node, Expr, Modifier};
use pagelistbot_solver_core::{Solver, Error};
use pagelistbot_provider_core::{PagePair, DataProvider};

pub struct RecursiveTreeSolver {
    provider: Box<dyn DataProvider>,
    default_limit: i64,
}

#[async_trait::async_trait]
impl<'a> Solver<'a> for RecursiveTreeSolver {

    async fn solve(&'a self, ast: &'a Node) -> Result<(BTreeSet<Title>, Vec<Error<'a>>), Error<'a>> {
        let (set, warnings) = self.solve_internal(ast).await?;
        Ok((
            BTreeSet::from_iter(set.into_iter().map(|(i, _)| i.title)),
            warnings
        ))
    }

}

impl<'a> RecursiveTreeSolver {

    pub fn new(provider: Box<dyn DataProvider>, default_limit: i64) -> Self {
        Self { provider, default_limit }
    }

    fn convert_modifier(&self, modifier: &'a Modifier) -> Modifier {
        let limit_converter = |lim: Option<i64>| {
            let lim = lim.unwrap_or(self.default_limit);
            if lim >= 0 {
                Some(lim)
            } else {
                None
            }
        };

        Modifier {
            result_limit: limit_converter(modifier.result_limit),
            ..modifier.to_owned()
        }
    }

    #[async_recursion::async_recursion]
    async fn solve_internal(&'a self, node: &'a Node) -> Result<(BTreeSet<PagePair>, Vec<Error<'a>>), Error<'a>> {
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

    async fn handle_page(&'a self, node: &'a Node, titles: &'a BTreeSet<String>) -> Result<(BTreeSet<PagePair>, Vec<Error<'a>>), Error<'a>> {
        let result = self.provider.get_page_info(titles).await;
        match result {
            Ok((infos, warnings)) => {
                let infos = BTreeSet::from_iter(infos.into_iter());
                let warnings: Vec<Error> = warnings.into_iter().map(|w| Error { node, content: w }).collect();
                Ok((infos, warnings))
            },
            Err(err) => Err(Error { node, content: err }),
        }
    }

    async fn handle_link(&'a self, node: &'a Node, target: &'a Node, modifier: &'a Modifier) -> Result<(BTreeSet<PagePair>, Vec<Error<'a>>), Error<'a>> {
        let (titles, mut warnings) = self.solve_internal(node).await?;
        let titles = BTreeSet::from_iter(titles.into_iter().map(|(i, _)| i.title));
        let result = self.provider.get_links(&titles, &self.convert_modifier(modifier)).await;
        match result {
            Ok((infos, new_warnings)) => {
                let infos = BTreeSet::from_iter(infos.into_iter());
                warnings.extend(new_warnings.into_iter().map(|w| Error { node, content: w }));
                Ok((infos, warnings))
            },
            Err(err) => Err(Error { node, content: err }),
        }
    }

    async fn handle_backlink(&'a self, node: &'a Node, target: &'a Node, modifier: &'a Modifier) -> Result<(BTreeSet<PagePair>, Vec<Error<'a>>), Error<'a>> {
        let (titles, mut warnings) = self.solve_internal(node).await?;
        let titles = BTreeSet::from_iter(titles.into_iter().map(|(i, _)| i.title));
        let result = self.provider.get_backlinks(&titles, &self.convert_modifier(modifier)).await;
        match result {
            Ok((infos, new_warnings)) => {
                let infos = BTreeSet::from_iter(infos.into_iter());
                warnings.extend(new_warnings.into_iter().map(|w| Error { node, content: w }));
                Ok((infos, warnings))
            },
            Err(err) => Err(Error { node, content: err }),
        }
    }

    async fn handle_embed(&'a self, node: &'a Node, target: &'a Node, modifier: &'a Modifier) -> Result<(BTreeSet<PagePair>, Vec<Error<'a>>), Error<'a>> {
        let (titles, mut warnings) = self.solve_internal(node).await?;
        let titles = BTreeSet::from_iter(titles.into_iter().map(|(i, _)| i.title));
        let result = self.provider.get_embeds(&titles, &self.convert_modifier(modifier)).await;
        match result {
            Ok((infos, new_warnings)) => {
                let infos = BTreeSet::from_iter(infos.into_iter());
                warnings.extend(new_warnings.into_iter().map(|w| Error { node, content: w }));
                Ok((infos, warnings))
            },
            Err(err) => Err(Error { node, content: err }),
        }
    }

    async fn handle_incategory(&'a self, node: &'a Node, target: &'a Node, modifier: &Modifier) -> Result<(BTreeSet<PagePair>, Vec<Error<'a>>), Error<'a>> {
        let (titles, mut warnings) = self.solve_internal(node).await?;
        let titles = BTreeSet::from_iter(titles.into_iter().map(|(i, _)| i.title));
        let result = self.provider.get_category_members(&titles, &self.convert_modifier(modifier)).await;
        match result {
            Ok((infos, new_warnings)) => {
                let infos = BTreeSet::from_iter(infos.into_iter());
                warnings.extend(new_warnings.into_iter().map(|w| Error { node, content: w }));
                Ok((infos, warnings))
            },
            Err(err) => Err(Error { node, content: err }),
        }
    }

    async fn handle_prefix(&'a self, node: &'a Node, target: &'a Node, modifier: &Modifier) -> Result<(BTreeSet<PagePair>, Vec<Error<'a>>), Error<'a>> {
        let (titles, mut warnings) = self.solve_internal(node).await?;
        let titles = BTreeSet::from_iter(titles.into_iter().map(|(i, _)| i.title));
        let result = self.provider.get_prefix(&titles, &self.convert_modifier(modifier)).await;
        match result {
            Ok((infos, new_warnings)) => {
                let infos = BTreeSet::from_iter(infos.into_iter());
                warnings.extend(new_warnings.into_iter().map(|w| Error { node, content: w }));
                Ok((infos, warnings))
            },
            Err(err) => Err(Error { node, content: err }),
        }
    }

    async fn handle_toggle(&'a self, node: &'a Node, target: &'a Node) -> Result<(BTreeSet<PagePair>, Vec<Error<'a>>), Error<'a>> {
        let (mut set, warnings) = self.solve_internal(target).await?;
        // No page's associated page lies in virtual-namespaces.
        // If so, we assert that the associated page should not exist at all (`Bad Title`, eg. no `Topic talk` namespace).
        set.retain(|(_, asso)| asso.title.namespace() >= 0);
        let toggled_set = BTreeSet::from_iter(set.into_iter().map(|(subj, asso)| (asso, subj)));
        Ok((toggled_set, warnings))
    }

    async fn handle_intersection(&'a self, node: &'a Node, set1: &'a Node, set2: &'a Node) -> Result<(BTreeSet<PagePair>, Vec<Error<'a>>), Error<'a>> {
        let (set1_result, set1_warnings) = self.solve_internal(set1).await?;
        let (set2_result, set2_warnings) = self.solve_internal(set2).await?;
        let intersection = set1_result.intersection(&set2_result);
        let set = BTreeSet::from_iter(intersection.cloned().collect::<BTreeSet<PagePair>>());
        let mut warnings = Vec::new();
        warnings.extend(set1_warnings);
        warnings.extend(set2_warnings);
        Ok((set, warnings))
    }

    async fn handle_xor(&'a self, node: &'a Node, set1: &'a Node, set2: &'a Node) -> Result<(BTreeSet<PagePair>, Vec<Error<'a>>), Error<'a>> {
        let (set1_result, set1_warnings) = self.solve_internal(set1).await?;
        let (set2_result, set2_warnings) = self.solve_internal(set2).await?;
        let intersection = set1_result.symmetric_difference(&set2_result);
        let set = BTreeSet::from_iter(intersection.cloned().collect::<BTreeSet<PagePair>>());
        let mut warnings = Vec::new();
        warnings.extend(set1_warnings);
        warnings.extend(set2_warnings);
        Ok((set, warnings))
    }

    async fn handle_difference(&'a self, node: &'a Node, set1: &'a Node, set2: &'a Node) -> Result<(BTreeSet<PagePair>, Vec<Error<'a>>), Error<'a>> {
        let (set1_result, set1_warnings) = self.solve_internal(set1).await?;
        let (set2_result, set2_warnings) = self.solve_internal(set2).await?;
        let intersection = set1_result.difference(&set2_result);
        let set = BTreeSet::from_iter(intersection.cloned().collect::<BTreeSet<PagePair>>());
        let mut warnings = Vec::new();
        warnings.extend(set1_warnings);
        warnings.extend(set2_warnings);
        Ok((set, warnings))
    }

    async fn handle_union(&'a self, node: &'a Node, set1: &'a Node, set2: &'a Node) -> Result<(BTreeSet<PagePair>, Vec<Error<'a>>), Error<'a>> {
        let (set1_result, set1_warnings) = self.solve_internal(set1).await?;
        let (set2_result, set2_warnings) = self.solve_internal(set2).await?;
        let intersection = set1_result.union(&set2_result);
        let set = BTreeSet::from_iter(intersection.cloned().collect::<BTreeSet<PagePair>>());
        let mut warnings = Vec::new();
        warnings.extend(set1_warnings);
        warnings.extend(set2_warnings);
        Ok((set, warnings))
    }

}
