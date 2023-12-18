//! Streams for query execution

use ast::{Span, Expression};
use async_stream::stream;
use mwtitle::Title;
use core::mem;
use crate::{SolverResult, RuntimeError, RuntimeWarning, SemanticError, attr::*};
use futures::{Stream, StreamExt};
use intorinf::IntOrInf;
use provider::DataProvider;
use std::collections::BTreeSet;
use trio_result::TrioResult;

/// Make the output unique.
fn unique<I, P>(stream: I, span: Span) -> impl Stream<Item=SolverResult<P>>
where
    I: Stream<Item=SolverResult<P>>,
    P: DataProvider,
{
    stream! {
        let mut yielded = BTreeSet::new();
        for await input in stream {
            match input {
                TrioResult::Ok(info) => {
                    let t = match info.get_title() {
                        Ok(x) => x,
                        Err(e) => {
                            yield TrioResult::Err(RuntimeError::PageInfo { span, error: e });
                            continue;
                        },
                    };
                    if !yielded.contains(t) {
                        yielded.insert(t.to_owned());
                        yield TrioResult::Ok(info);
                    }
                },
                x => yield x,
            }
        }
    }
}

/// Make the output counted.
fn counted<I, P>(stream: I, limit: usize, span: Span) -> impl Stream<Item=SolverResult<P>>
where
    I: Stream<Item=SolverResult<P>>,
    P: DataProvider,
{
    stream! {
        let mut count = 0;
        for await item in stream {
            match item {
                x @ TrioResult::Ok(_) => {
                    count += 1;
                    if count <= limit {
                        yield x;
                    } else {
                        yield TrioResult::Warn(RuntimeWarning::ResultLimitExceeded { span, limit });
                        break;
                    }
                },
                x => yield x,
            }
        }
    }
}

/// After the first error, the stream is cut and no longer returns anything.
fn cut<I, P>(stream: I) -> impl Stream<Item=SolverResult<P>>
where
    I: Stream<Item=SolverResult<P>>,
    P: DataProvider,
{
    stream! {
        for await item in stream {
            match item {
                x @ TrioResult::Err(_) => {
                    yield x;
                    break;
                },
                x => yield x,
            }
        }
    }
}

/// Raw page info stream.
fn pageinfo<I, P>(titles: I, provider: P, span: Span) -> impl Stream<Item=SolverResult<P>>
where
    I: IntoIterator<Item=String>,
    P: DataProvider,
{
    stream! {
        let st = provider.get_page_info_from_raw(titles);
        for await item in st {
            match item {
                TrioResult::Ok(item) => yield TrioResult::Ok(item),
                TrioResult::Warn(w) => yield TrioResult::Warn(RuntimeWarning::Provider { span, warn: w }),
                TrioResult::Err(e) => yield TrioResult::Err(RuntimeError::Provider { span, error: e }),
            }
        }
    }
}

macro_rules! make_query {
    ($method:ident, $trait_method:ident, $config_class:ty) => {
        /// Make a normal query stream.
        fn $method<I, P>(stream: I, provider: P, config: $config_class, span: ast::Span) -> impl Stream<Item=SolverResult<P>>
        where
            I: Stream<Item=SolverResult<P>>,
            P: DataProvider,
        {
            stream! {
                for await i in stream {
                    if let TrioResult::Ok(i) = i {
                        // make stream
                        let t = match i.try_into() {
                            Ok(t) => t,
                            Err(w) => {
                                yield TrioResult::Err(RuntimeError::PageInfo { span, error: w });
                                continue;
                            }
                        };
                        let st = provider.$trait_method(t, &config);
                        // poll stream
                        for await item in st {
                            match item {
                                TrioResult::Ok(item) => yield TrioResult::Ok(item),
                                TrioResult::Warn(w) => yield TrioResult::Warn(RuntimeWarning::Provider { span, warn: w }),
                                TrioResult::Err(e) => yield TrioResult::Err(RuntimeError::Provider { span, error: e }),
                            }
                        }
                    } else {
                        // yield any warnings or errors
                        yield i;
                    }
                }
            }
        }
    };
}

make_query!(links, get_links, provider::LinksConfig);
make_query!(backlinks, get_backlinks, provider::BackLinksConfig);
make_query!(embeds, get_embeds, provider::EmbedsConfig);
make_query!(prefix, get_prefix, provider::PrefixConfig);

// Make a category member stream.
fn categorymembers<I, P>(stream: I, provider: P, config: provider::CategoryMembersConfig, max_depth: IntOrInf, span: Span) -> impl Stream<Item=SolverResult<P>>
where
    I: Stream<Item=SolverResult<P>>,
    P: DataProvider,
{
    stream! {
        for await t in stream {
            if let TrioResult::Ok(t) = t {
                // search based on this title.
                let t: Title = match t.try_into() {
                    Ok(t) => t,
                    Err(e) => {
                        yield TrioResult::Err(RuntimeError::PageInfo { span, error: e });
                        continue;
                    }
                };
                let mut current_depth = IntOrInf::Int(0);
                let mut visited_categories = BTreeSet::new();
                let mut to_visit = BTreeSet::new();

                to_visit.insert(t.clone());
                visited_categories.insert(t);

                while !to_visit.is_empty() {
                    // prepare configuration, add category namespace is recursive.
                    let mut query_config = config.clone();
                    if current_depth < max_depth {
                        if let Some(ns) = &mut query_config.namespace {
                            ns.insert(14);
                        }
                    }
                    // prepare stream
                    let queue = mem::take(&mut to_visit);
                    let stream = provider.get_category_members_multi(queue, &query_config);
                    // poll stream
                    for await i in stream {
                        match i {
                            TrioResult::Ok(item) => {
                                // some category members...
                                let t = match item.get_title() {
                                    Ok(t) => t,
                                    Err(e) => {
                                        yield TrioResult::Err(RuntimeError::PageInfo { span, error: e });
                                        continue;
                                    }
                                };
                                // add to visit queue?
                                if t.is_category() && !visited_categories.contains(t) && current_depth < max_depth {
                                    to_visit.insert(t.to_owned());
                                    visited_categories.insert(t.to_owned());
                                }
                                // yield this item?
                                if !config.namespace.as_ref().is_some_and(|ns| !ns.contains(&t.namespace())) {
                                    yield TrioResult::Ok(item);
                                }
                            },
                            TrioResult::Warn(w) => yield TrioResult::Warn(RuntimeWarning::Provider { span, warn: w }),
                            TrioResult::Err(e) => yield TrioResult::Err(RuntimeError::Provider { span, error: e }),
                        }
                    }
                    // end of this layer.
                    current_depth += 1;
                }
            } else {
                // yield any warnings or errors
                yield t;
            }
        }
    }
}

/// Make a toggle stream that swaps the page with its associated page.
fn toggle<I, P>(stream: I, span: Span) -> impl Stream<Item = SolverResult<P>>
where
    I: Stream<Item = SolverResult<P>>,
    P: DataProvider,
{
    stream! {
        for await item in stream {
            if let TrioResult::Ok(mut item) = item {
                item.swap();

                // TODO: do we still need this?
                // No page's associated page lies in virtual namespaces.
                // If so, we assert that the associated page should not exist at all (`Bad Title`, eg. no `Topic talk` namespace).
                // In such case, we do not yield this item, the loop goes on.
                let t = match item.get_title() {
                    Ok(t) => t,
                    Err(e) => {
                        yield TrioResult::Err(RuntimeError::PageInfo { span, error: e });
                        continue;
                    },
                };
                if t.namespace() >= 0 {
                    yield TrioResult::Ok(item);
                }
            }
        }
    }
}

macro_rules! set_operation {
    ($method:ident, $op:path) => {
        /// Make a set operation stream.
        fn $method<I1, I2, P>(stream1: I1, stream2: I2) -> impl Stream<Item = SolverResult<P>>
        where
            I1: Stream<Item = SolverResult<P>>, // + core::marker::Unpin,
            I2: Stream<Item = SolverResult<P>>, // + core::marker::Unpin,
            P: DataProvider,
        {
            stream! {
                let st1 = Box::pin(stream1.map(|x| (x, false)));
                let st2 = Box::pin(stream2.map(|x| (x, true)));
                let combined = futures::stream_select!(st1, st2);
                let mut set1 = BTreeSet::new();
                let mut set2 = BTreeSet::new();

                for await item in combined {
                    match item {
                        (TrioResult::Ok(item), false) => { set1.insert(item); },
                        (TrioResult::Ok(item), true) => { set2.insert(item); },
                        (x, _) => { yield x; },
                    }
                }

                for item in $op(&set1, &set2) {
                    yield TrioResult::Ok(item.to_owned());
                }
            }
        }
    }
}

set_operation!(set_intersection, BTreeSet::intersection);
set_operation!(set_union, BTreeSet::union);
set_operation!(set_difference, BTreeSet::difference);
set_operation!(set_xor, BTreeSet::symmetric_difference);

/// Create a stream from an expression.
pub fn from_expr<'a, P>(expr: &Expression, provider: P, default_count_limit: IntOrInf) -> Result<Box<dyn Stream<Item=SolverResult<P>> + 'a>, SemanticError>
where
    P: DataProvider + Clone + 'a,
{
    let st = from_expr_inner(expr, provider, default_count_limit)?;
    Ok(Box::new(cut(Box::into_pin(st))))
}

fn from_expr_inner<'a, P>(expr: &Expression, provider: P, default_count_limit: IntOrInf) -> Result<Box<dyn Stream<Item=SolverResult<P>> + 'a>, SemanticError>
where
    P: DataProvider + Clone + 'a,
{
    match expr {
        Expression::And(expr) => {
            let st1 = from_expr_inner(&expr.expr1, provider.clone(), default_count_limit)?;
            let st2 = from_expr_inner(&expr.expr2, provider.clone(), default_count_limit)?;
            Ok(Box::new(set_intersection(Box::into_pin(st1), Box::into_pin(st2))))
        },
        Expression::Add(expr) => {
            let st1 = from_expr_inner(&expr.expr1, provider.clone(), default_count_limit)?;
            let st2 = from_expr_inner(&expr.expr2, provider.clone(), default_count_limit)?;
            Ok(Box::new(set_union(Box::into_pin(st1), Box::into_pin(st2))))
        },
        Expression::Sub(expr) => {
            let st1 = from_expr_inner(&expr.expr1, provider.clone(), default_count_limit)?;
            let st2 = from_expr_inner(&expr.expr2, provider.clone(), default_count_limit)?;
            Ok(Box::new(set_difference(Box::into_pin(st1), Box::into_pin(st2))))
        },
        Expression::Xor(expr) => {
            let st1 = from_expr_inner(&expr.expr1, provider.clone(), default_count_limit)?;
            let st2 = from_expr_inner(&expr.expr2, provider.clone(), default_count_limit)?;
            Ok(Box::new(set_xor(Box::into_pin(st1), Box::into_pin(st2))))
        },
        Expression::Paren(expr) => {
            from_expr_inner(&expr.expr, provider, default_count_limit)
        },
        Expression::Page(expr) => {
            let pages: Vec<_> = expr.vals.iter().map(|lit| lit.val.to_owned()).collect();
            Ok(Box::new(pageinfo(pages, provider, expr.get_span())))
        },
        Expression::Link(expr) => {
            let (config, limit) = links_config_from_attributes(&expr.attributes)?;
            let mut st = from_expr_inner(&expr.expr, provider.clone(), default_count_limit)?;
            st = Box::new(links(Box::into_pin(st), provider, config, expr.get_span()));
            if limit.is_some_and(|l| l.is_int()) || (limit.is_none() && default_count_limit.is_int()) {
                st = Box::new(counted(Box::into_pin(st), limit.unwrap_or(default_count_limit).unwrap_int() as usize, expr.get_span()))
            }
            Ok(Box::new(unique(Box::into_pin(st), expr.get_span())))
        },
        Expression::LinkTo(expr) => {
            let (config, limit) = backlinks_config_from_attributes(&expr.attributes)?;
            let mut st = from_expr_inner(&expr.expr, provider.clone(), default_count_limit)?;
            st = Box::new(backlinks(Box::into_pin(st), provider, config, expr.get_span()));
            if limit.is_some_and(|l| l.is_int()) || (limit.is_none() && default_count_limit.is_int()) {
                st = Box::new(counted(Box::into_pin(st), limit.unwrap_or(default_count_limit).unwrap_int() as usize, expr.get_span()))
            }
            Ok(Box::new(unique(Box::into_pin(st), expr.get_span())))
        },
        Expression::Embed(expr) => {
            let (config, limit) = embeds_config_from_attributes(&expr.attributes)?;
            let mut st = from_expr_inner(&expr.expr, provider.clone(), default_count_limit)?;
            st = Box::new(embeds(Box::into_pin(st), provider, config, expr.get_span()));
            if limit.is_some_and(|l| l.is_int()) || (limit.is_none() && default_count_limit.is_int()) {
                st = Box::new(counted(Box::into_pin(st), limit.unwrap_or(default_count_limit).unwrap_int() as usize, expr.get_span()))
            }
            Ok(Box::new(unique(Box::into_pin(st), expr.get_span())))
        },
        Expression::InCat(expr) => {
            let (config, limit, depth) = categorymembers_config_from_attributes(&expr.attributes)?;
            let mut st = from_expr_inner(&expr.expr, provider.clone(), default_count_limit)?;
            st = Box::new(categorymembers(Box::into_pin(st), provider, config, depth.unwrap_or(IntOrInf::Int(0)), expr.get_span()));
            if limit.is_some_and(|l| l.is_int()) || (limit.is_none() && default_count_limit.is_int()) {
                st = Box::new(counted(Box::into_pin(st), limit.unwrap_or(default_count_limit).unwrap_int() as usize, expr.get_span()))
            }
            Ok(Box::new(unique(Box::into_pin(st), expr.get_span())))
        },
        Expression::Prefix(expr) => {
            let (config, limit) = prefix_config_from_attributes(&expr.attributes)?;
            let mut st = from_expr_inner(&expr.expr, provider.clone(), default_count_limit)?;
            st = Box::new(prefix(Box::into_pin(st), provider, config, expr.get_span()));
            if limit.is_some_and(|l| l.is_int()) || (limit.is_none() && default_count_limit.is_int()) {
                st = Box::new(counted(Box::into_pin(st), limit.unwrap_or(default_count_limit).unwrap_int() as usize, expr.get_span()))
            }
            Ok(Box::new(unique(Box::into_pin(st), expr.get_span())))
        },
        Expression::Toggle(expr) => {
            let st = from_expr_inner(&expr.expr, provider, default_count_limit)?;
            Ok(Box::new(toggle(Box::into_pin(st), expr.get_span())))
        },
        _ => unimplemented!(),
    }
}
