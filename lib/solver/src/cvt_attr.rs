//! Convert attributes to configs.

use ast::{Attribute, Modifier, Span};
use core::fmt;
use crate::core::{Solver, SolverError};
use intorinf::IntOrInf;
use provider::{
    FilterRedirect,
    LinksConfig, BackLinksConfig, EmbedsConfig, CategoryMembersConfig, PrefixConfig,
};
use std::{collections::{HashSet, HashMap}, error::Error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeError {
    /// This attribute conflicts with another attribute.
    Conflict(Span),
    /// This attribute is a duplicate of another attribute.
    Duplicate(Span),
    /// This attribute is invalid under this operation.
    Invalid,
}

impl fmt::Display for AttributeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttributeError::Conflict(_) => write!(f, "attribute conflicts with another attribute"),
            AttributeError::Duplicate(_) => write!(f, "attribute duplicates with another attribute"),
            AttributeError::Invalid => write!(f, "invalid attribute"),
        }
    }
}

impl Error for AttributeError {}
// unsafe impl<'e> Send for AttributeError<'e> {}

/// Convert a collection of `Attribute`s into a `LinksConfig` and a limit.
pub fn links_config_from_attributes<S>(attrs: &[Attribute]) -> Result<(LinksConfig, Option<IntOrInf>), SolverError<S>>
where
    S: Solver,
{
    // core things
    let mut config = LinksConfig::default();
    let mut limit: Option<IntOrInf> = None;
    // resolved at objects.
    let mut resolved_at: HashMap<&str, Span> = HashMap::new();
    for attr in attrs {
        if let Attribute::Modifier(attr) = attr {
            match &attr.modifier {
                Modifier::Limit(item) => {
                    if let Some(span) = resolved_at.get("limit") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Duplicate(span.to_owned())));
                    } else {
                        resolved_at.insert("limit", item.get_span().to_owned());
                        limit = Some(item.val.val);
                    }
                },
                Modifier::Resolve(item) => {
                    if let Some(span) = resolved_at.get("resolve") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Duplicate(span.to_owned())));
                    } else {
                        resolved_at.insert("resolve", item.get_span().to_owned());
                        config.resolve_redirects = true;
                    }
                },
                Modifier::Ns(item) => {
                    if let Some(span) = resolved_at.get("ns") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Duplicate(span.to_owned())));
                    } else {
                        resolved_at.insert("ns", item.get_span().to_owned());
                        let namespace = item.vals.iter().map(|lit| lit.val).collect::<HashSet<_>>();
                        config.namespace = Some(namespace);
                    }
                },
                _ => {
                    return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Invalid));
                },
            }
        }
    }
    Ok((config, limit))
}

/// Convert a collection of `Attribute`s into a `BackLinksConfig` and a limit.
pub fn backlinks_config_from_attributes<S>(attrs: &[Attribute]) -> Result<(BackLinksConfig, Option<IntOrInf>), SolverError<S>>
where
    S: Solver,
{
    // core things
    let mut config = BackLinksConfig::default();
    let mut limit: Option<IntOrInf> = None;
    // resolved at objects.
    let mut resolved_at: HashMap<&str, Span> = HashMap::new();
    for attr in attrs {
        if let Attribute::Modifier(attr) = attr {
            match &attr.modifier {
                Modifier::Limit(item) => {
                    if let Some(span) = resolved_at.get("limit") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Duplicate(span.to_owned())));
                    } else {
                        resolved_at.insert("limit", item.get_span().to_owned());
                        limit = Some(item.val.val);
                    }
                },
                Modifier::Resolve(item) => {
                    if let Some(span) = resolved_at.get("resolve") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Duplicate(span.to_owned())));
                    } else {
                        resolved_at.insert("resolve", item.get_span().to_owned());
                        config.resolve_redirects = true;
                    }
                },
                Modifier::Ns(item) => {
                    if let Some(span) = resolved_at.get("ns") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Duplicate(span.to_owned())));
                    } else {
                        resolved_at.insert("ns", item.get_span().to_owned());
                        let namespace = item.vals.iter().map(|lit| lit.val).collect::<HashSet<_>>();
                        config.namespace = Some(namespace);
                    }
                },
                Modifier::NoRedir(item) => {
                    if let Some(span) = resolved_at.get("noredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Duplicate(span.to_owned())));
                    } else if let Some(span) = resolved_at.get("onlyredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Conflict(span.to_owned())));
                    } else {
                        resolved_at.insert("noredir", item.get_span().to_owned());
                        config.filter_redirects = Some(FilterRedirect::NoRedirect);
                    }
                },
                Modifier::OnlyRedir(item) => {
                    if let Some(span) = resolved_at.get("onlyredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Duplicate(span.to_owned())));
                    } else if let Some(span) = resolved_at.get("noredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Conflict(span.to_owned())));
                    } else {
                        resolved_at.insert("onlyredir", item.get_span().to_owned());
                        config.filter_redirects = Some(FilterRedirect::OnlyRedirect);
                    }
                },
                Modifier::Direct(item) => {
                    if let Some(span) = resolved_at.get("direct") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Duplicate(span.to_owned())));
                    } else {
                        resolved_at.insert("direct", item.get_span().to_owned());
                        config.direct = true;
                    }
                },
                _ => {
                    return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Invalid));
                },
            }
        }
    }
    Ok((config, limit))
}

/// Convert a collection of `Attribute`s into a `EmbedsConfig` and a limit.
pub fn embeds_config_from_attributes<S>(attrs: &[Attribute]) -> Result<(EmbedsConfig, Option<IntOrInf>), SolverError<S>>
where
    S: Solver,
{
    // core things
    let mut config = EmbedsConfig::default();
    let mut limit: Option<IntOrInf> = None;
    // resolved at objects.
    let mut resolved_at: HashMap<&str, Span> = HashMap::new();
    for attr in attrs {
        if let Attribute::Modifier(attr) = attr {
            match &attr.modifier {
                Modifier::Limit(item) => {
                    if let Some(span) = resolved_at.get("limit") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Duplicate(span.to_owned())));
                    } else {
                        resolved_at.insert("limit", item.get_span().to_owned());
                        limit = Some(item.val.val);
                    }
                },
                Modifier::Resolve(item) => {
                    if let Some(span) = resolved_at.get("resolve") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Duplicate(span.to_owned())));
                    } else {
                        resolved_at.insert("resolve", item.get_span().to_owned());
                        config.resolve_redirects = true;
                    }
                },
                Modifier::Ns(item) => {
                    if let Some(span) = resolved_at.get("ns") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Duplicate(span.to_owned())));
                    } else {
                        resolved_at.insert("ns", item.get_span().to_owned());
                        let namespace = item.vals.iter().map(|lit| lit.val).collect::<HashSet<_>>();
                        config.namespace = Some(namespace);
                    }
                },
                Modifier::NoRedir(item) => {
                    if let Some(span) = resolved_at.get("noredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Duplicate(span.to_owned())));
                    } else if let Some(span) = resolved_at.get("onlyredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Conflict(span.to_owned())));
                    } else {
                        resolved_at.insert("noredir", item.get_span().to_owned());
                        config.filter_redirects = Some(FilterRedirect::NoRedirect);
                    }
                },
                Modifier::OnlyRedir(item) => {
                    if let Some(span) = resolved_at.get("onlyredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Duplicate(span.to_owned())));
                    } else if let Some(span) = resolved_at.get("noredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Conflict(span.to_owned())));
                    } else {
                        resolved_at.insert("onlyredir", item.get_span().to_owned());
                        config.filter_redirects = Some(FilterRedirect::OnlyRedirect);
                    }
                },
                _ => {
                    return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Invalid));
                },
            }
        }
    }
    Ok((config, limit))
}

/// Convert a collection of `Attribute`s into a `CategoryMembersConfig` and a limit and a depth.
pub fn categorymembers_config_from_attributes<S>(attrs: &[Attribute]) -> Result<(CategoryMembersConfig, Option<IntOrInf>, Option<IntOrInf>), SolverError<S>>
where
    S: Solver,
{
    // core things
    let mut config = CategoryMembersConfig::default();
    let mut limit: Option<IntOrInf> = None;
    let mut depth: Option<IntOrInf> = None;
    // resolved at objects.
    let mut resolved_at: HashMap<&str, Span> = HashMap::new();
    for attr in attrs {
        if let Attribute::Modifier(attr) = attr {
            match &attr.modifier {
                Modifier::Limit(item) => {
                    if let Some(span) = resolved_at.get("limit") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Duplicate(span.to_owned())));
                    } else {
                        resolved_at.insert("limit", item.get_span().to_owned());
                        limit = Some(item.val.val);
                    }
                },
                Modifier::Resolve(item) => {
                    if let Some(span) = resolved_at.get("resolve") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Duplicate(span.to_owned())));
                    } else {
                        resolved_at.insert("resolve", item.get_span().to_owned());
                        config.resolve_redirects = true;
                    }
                },
                Modifier::Ns(item) => {
                    if let Some(span) = resolved_at.get("ns") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Duplicate(span.to_owned())));
                    } else {
                        resolved_at.insert("ns", item.get_span().to_owned());
                        let namespace = item.vals.iter().map(|lit| lit.val).collect::<HashSet<_>>();
                        config.namespace = Some(namespace);
                    }
                },
                Modifier::Depth(item) => {
                    if let Some(span) = resolved_at.get("depth") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Duplicate(span.to_owned())));
                    } else {
                        resolved_at.insert("depth", item.get_span().to_owned());
                        depth = Some(item.val.val);
                    }
                },
                _ => {
                    return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Invalid));
                },
            }
        }
    }
    Ok((config, limit, depth))
}

/// Convert a collection of `Attribute`s into a `PrefixConfig` and a limit.
pub fn prefix_config_from_attributes<S>(attrs: &[Attribute]) -> Result<(PrefixConfig, Option<IntOrInf>), SolverError<S>>
where
    S: Solver,
{
    // core things
    let mut config = PrefixConfig::default();
    let mut limit: Option<IntOrInf> = None;
    // resolved at objects.
    let mut resolved_at: HashMap<&str, Span> = HashMap::new();
    for attr in attrs {
        if let Attribute::Modifier(attr) = attr {
            match &attr.modifier {
                Modifier::Limit(item) => {
                    if let Some(span) = resolved_at.get("limit") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Duplicate(span.to_owned())));
                    } else {
                        resolved_at.insert("limit", item.get_span().to_owned());
                        limit = Some(item.val.val);
                    }
                },
                Modifier::NoRedir(item) => {
                    if let Some(span) = resolved_at.get("noredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Duplicate(span.to_owned())));
                    } else if let Some(span) = resolved_at.get("onlyredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Conflict(span.to_owned())));
                    } else {
                        resolved_at.insert("noredir", item.get_span().to_owned());
                        config.filter_redirects = Some(FilterRedirect::NoRedirect);
                    }
                },
                Modifier::OnlyRedir(item) => {
                    if let Some(span) = resolved_at.get("onlyredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Duplicate(span.to_owned())));
                    } else if let Some(span) = resolved_at.get("noredir") {
                        return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Conflict(span.to_owned())));
                    } else {
                        resolved_at.insert("onlyredir", item.get_span().to_owned());
                        config.filter_redirects = Some(FilterRedirect::OnlyRedirect);
                    }
                },
                _ => {
                    return Err(SolverError::from_attribute_error(attr.get_span().to_owned(), AttributeError::Invalid));
                },
            }
        }
    }
    Ok((config, limit))
}
