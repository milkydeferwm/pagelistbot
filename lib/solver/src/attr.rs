//! Convert attributes to configs.

use ast::{Attribute, Modifier, Span};
use crate::SemanticError;
use intorinf::IntOrInf;
use provider::{
    FilterRedirect,
    LinksConfig, BackLinksConfig, EmbedsConfig, CategoryMembersConfig, PrefixConfig,
};
use std::collections::{HashSet, HashMap};

/// Convert a collection of `Attribute`s into a `LinksConfig` and a limit.
pub fn links_config_from_attributes(attrs: &[Attribute]) -> Result<(LinksConfig, Option<IntOrInf>), SemanticError> {
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
                        return Err(SemanticError::DuplicateAttribute { span: attr.get_span(), other: *span });
                    } else {
                        resolved_at.insert("limit", attr.get_span());
                        limit = Some(item.val.val);
                    }
                },
                Modifier::Resolve(item) => {
                    if let Some(span) = resolved_at.get("resolve") {
                        return Err(SemanticError::DuplicateAttribute { span: attr.get_span(), other: *span });
                    } else {
                        resolved_at.insert("resolve", item.get_span());
                        config.resolve_redirects = true;
                    }
                },
                Modifier::Ns(item) => {
                    if let Some(span) = resolved_at.get("ns") {
                        return Err(SemanticError::DuplicateAttribute { span: attr.get_span(), other: *span });
                    } else {
                        resolved_at.insert("ns", item.get_span());
                        let namespace = item.vals.iter().map(|lit| lit.val).collect::<HashSet<_>>();
                        config.namespace = Some(namespace);
                    }
                },
                _ => {
                    return Err(SemanticError::InvalidAttribute { span: attr.get_span() });
                },
            }
        }
    }
    Ok((config, limit))
}

/// Convert a collection of `Attribute`s into a `BackLinksConfig` and a limit.
pub fn backlinks_config_from_attributes(attrs: &[Attribute]) -> Result<(BackLinksConfig, Option<IntOrInf>), SemanticError> {
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
                        return Err(SemanticError::DuplicateAttribute { span: attr.get_span(), other: *span });
                    } else {
                        resolved_at.insert("limit", item.get_span());
                        limit = Some(item.val.val);
                    }
                },
                Modifier::Resolve(item) => {
                    if let Some(span) = resolved_at.get("resolve") {
                        return Err(SemanticError::DuplicateAttribute { span: attr.get_span(), other: *span });
                    } else {
                        resolved_at.insert("resolve", item.get_span());
                        config.resolve_redirects = true;
                    }
                },
                Modifier::Ns(item) => {
                    if let Some(span) = resolved_at.get("ns") {
                        return Err(SemanticError::DuplicateAttribute { span: attr.get_span(), other: *span });
                    } else {
                        resolved_at.insert("ns", item.get_span());
                        let namespace = item.vals.iter().map(|lit| lit.val).collect::<HashSet<_>>();
                        config.namespace = Some(namespace);
                    }
                },
                Modifier::NoRedir(item) => {
                    if let Some(span) = resolved_at.get("noredir") {
                        return Err(SemanticError::DuplicateAttribute { span: attr.get_span(), other: *span });
                    } else if let Some(span) = resolved_at.get("onlyredir") {
                        return Err(SemanticError::ConflictAttribute { span: attr.get_span(), other: *span });
                    } else {
                        resolved_at.insert("noredir", item.get_span());
                        config.filter_redirects = Some(FilterRedirect::NoRedirect);
                    }
                },
                Modifier::OnlyRedir(item) => {
                    if let Some(span) = resolved_at.get("onlyredir") {
                        return Err(SemanticError::DuplicateAttribute { span: attr.get_span(), other: *span });
                    } else if let Some(span) = resolved_at.get("noredir") {
                        return Err(SemanticError::ConflictAttribute { span: attr.get_span(), other: *span });
                    } else {
                        resolved_at.insert("onlyredir", item.get_span());
                        config.filter_redirects = Some(FilterRedirect::OnlyRedirect);
                    }
                },
                Modifier::Direct(item) => {
                    if let Some(span) = resolved_at.get("direct") {
                        return Err(SemanticError::DuplicateAttribute { span: attr.get_span(), other: *span });
                    } else {
                        resolved_at.insert("direct", item.get_span());
                        config.direct = true;
                    }
                },
                _ => {
                    return Err(SemanticError::InvalidAttribute { span: attr.get_span() });
                },
            }
        }
    }
    Ok((config, limit))
}

/// Convert a collection of `Attribute`s into a `EmbedsConfig` and a limit.
pub fn embeds_config_from_attributes(attrs: &[Attribute]) -> Result<(EmbedsConfig, Option<IntOrInf>), SemanticError> {
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
                        return Err(SemanticError::DuplicateAttribute { span: attr.get_span(), other: *span });
                    } else {
                        resolved_at.insert("limit", item.get_span());
                        limit = Some(item.val.val);
                    }
                },
                Modifier::Resolve(item) => {
                    if let Some(span) = resolved_at.get("resolve") {
                        return Err(SemanticError::DuplicateAttribute { span: attr.get_span(), other: *span });
                    } else {
                        resolved_at.insert("resolve", item.get_span());
                        config.resolve_redirects = true;
                    }
                },
                Modifier::Ns(item) => {
                    if let Some(span) = resolved_at.get("ns") {
                        return Err(SemanticError::DuplicateAttribute { span: attr.get_span(), other: *span });
                    } else {
                        resolved_at.insert("ns", item.get_span());
                        let namespace = item.vals.iter().map(|lit| lit.val).collect::<HashSet<_>>();
                        config.namespace = Some(namespace);
                    }
                },
                Modifier::NoRedir(item) => {
                    if let Some(span) = resolved_at.get("noredir") {
                        return Err(SemanticError::DuplicateAttribute { span: attr.get_span(), other: *span });
                    } else if let Some(span) = resolved_at.get("onlyredir") {
                        return Err(SemanticError::ConflictAttribute { span: attr.get_span(), other: *span });
                    } else {
                        resolved_at.insert("noredir", item.get_span());
                        config.filter_redirects = Some(FilterRedirect::NoRedirect);
                    }
                },
                Modifier::OnlyRedir(item) => {
                    if let Some(span) = resolved_at.get("onlyredir") {
                        return Err(SemanticError::DuplicateAttribute { span: attr.get_span(), other: *span });
                    } else if let Some(span) = resolved_at.get("noredir") {
                        return Err(SemanticError::ConflictAttribute { span: attr.get_span(), other: *span });
                    } else {
                        resolved_at.insert("onlyredir", item.get_span());
                        config.filter_redirects = Some(FilterRedirect::OnlyRedirect);
                    }
                },
                _ => {
                    return Err(SemanticError::InvalidAttribute { span: attr.get_span() });
                },
            }
        }
    }
    Ok((config, limit))
}

/// Convert a collection of `Attribute`s into a `CategoryMembersConfig` and a limit and a depth.
pub fn categorymembers_config_from_attributes(attrs: &[Attribute]) -> Result<(CategoryMembersConfig, Option<IntOrInf>, Option<IntOrInf>), SemanticError> {
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
                        return Err(SemanticError::DuplicateAttribute { span: attr.get_span(), other: *span });
                    } else {
                        resolved_at.insert("limit", item.get_span());
                        limit = Some(item.val.val);
                    }
                },
                Modifier::Resolve(item) => {
                    if let Some(span) = resolved_at.get("resolve") {
                        return Err(SemanticError::DuplicateAttribute { span: attr.get_span(), other: *span });
                    } else {
                        resolved_at.insert("resolve", item.get_span());
                        config.resolve_redirects = true;
                    }
                },
                Modifier::Ns(item) => {
                    if let Some(span) = resolved_at.get("ns") {
                        return Err(SemanticError::DuplicateAttribute { span: attr.get_span(), other: *span });
                    } else {
                        resolved_at.insert("ns", item.get_span());
                        let namespace = item.vals.iter().map(|lit| lit.val).collect::<HashSet<_>>();
                        config.namespace = Some(namespace);
                    }
                },
                Modifier::Depth(item) => {
                    if let Some(span) = resolved_at.get("depth") {
                        return Err(SemanticError::DuplicateAttribute { span: attr.get_span(), other: *span });
                    } else {
                        resolved_at.insert("depth", item.get_span());
                        depth = Some(item.val.val);
                    }
                },
                _ => {
                    return Err(SemanticError::InvalidAttribute { span: attr.get_span() });
                },
            }
        }
    }
    Ok((config, limit, depth))
}

/// Convert a collection of `Attribute`s into a `PrefixConfig` and a limit.
pub fn prefix_config_from_attributes(attrs: &[Attribute]) -> Result<(PrefixConfig, Option<IntOrInf>), SemanticError> {
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
                        return Err(SemanticError::DuplicateAttribute { span: attr.get_span(), other: *span });
                    } else {
                        resolved_at.insert("limit", item.get_span());
                        limit = Some(item.val.val);
                    }
                },
                Modifier::NoRedir(item) => {
                    if let Some(span) = resolved_at.get("noredir") {
                        return Err(SemanticError::DuplicateAttribute { span: attr.get_span(), other: *span });
                    } else if let Some(span) = resolved_at.get("onlyredir") {
                        return Err(SemanticError::ConflictAttribute { span: attr.get_span(), other: *span });
                    } else {
                        resolved_at.insert("noredir", item.get_span());
                        config.filter_redirects = Some(FilterRedirect::NoRedirect);
                    }
                },
                Modifier::OnlyRedir(item) => {
                    if let Some(span) = resolved_at.get("onlyredir") {
                        return Err(SemanticError::DuplicateAttribute { span: attr.get_span(), other: *span });
                    } else if let Some(span) = resolved_at.get("noredir") {
                        return Err(SemanticError::ConflictAttribute { span: attr.get_span(), other: *span });
                    } else {
                        resolved_at.insert("onlyredir", item.get_span());
                        config.filter_redirects = Some(FilterRedirect::OnlyRedirect);
                    }
                },
                _ => {
                    return Err(SemanticError::InvalidAttribute { span: attr.get_span() });
                },
            }
        }
    }
    Ok((config, limit))
}
