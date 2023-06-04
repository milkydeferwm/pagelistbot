use ast::Expression;
use crate::attr::*;
use crate::{SolverStream, SemanticError};
use crate::streams::*;
use intorinf::IntOrInf;
use provider::DataProvider;

impl<P: DataProvider + Clone> SolverStream<P> {
    pub fn from_expr(expr: &Expression, provider: P, default_count_limit: IntOrInf) -> Result<Self, SemanticError> {
        let st = Self::from_expr_inner(expr, provider, default_count_limit)?;
        // wrap in a cut-error wrapper.
        let wrapped = CutError::new(Box::pin(st));
        Ok(Self::CutError(wrapped))
    }

    fn from_expr_inner(expr: &Expression, provider: P, default_count_limit: IntOrInf) -> Result<Self, SemanticError> {
        match expr {
            Expression::And(expr) => {
                let st1 = Self::from_expr_inner(&expr.expr1, provider.clone(), default_count_limit)?;
                let st2 = Self::from_expr_inner(&expr.expr2, provider, default_count_limit)?;
                let st = IntersectionStream::new(Box::pin(st1), Box::pin(st2), expr.get_span());
                Ok(Self::Intersection(st))
            },
            Expression::Add(expr) => {
                let st1 = Self::from_expr_inner(&expr.expr1, provider.clone(), default_count_limit)?;
                let st2 = Self::from_expr_inner(&expr.expr2, provider, default_count_limit)?;
                let st = UnionStream::new(Box::pin(st1), Box::pin(st2), expr.get_span());
                Ok(Self::Union(st))
            },
            Expression::Sub(expr) => {
                let st1 = Self::from_expr_inner(&expr.expr1, provider.clone(), default_count_limit)?;
                let st2 = Self::from_expr_inner(&expr.expr2, provider, default_count_limit)?;
                let st = DifferenceStream::new(Box::pin(st1), Box::pin(st2), expr.get_span());
                Ok(Self::Difference(st))
            },
            Expression::Xor(expr) => {
                let st1 = Self::from_expr_inner(&expr.expr1, provider.clone(), default_count_limit)?;
                let st2 = Self::from_expr_inner(&expr.expr2, provider, default_count_limit)?;
                let st = XorStream::new(Box::pin(st1), Box::pin(st2), expr.get_span());
                Ok(Self::Xor(st))
            },
            Expression::Paren(expr) => {
                Self::from_expr_inner(&expr.expr, provider, default_count_limit)
            },
            Expression::Page(expr) => {
                let pages: Vec<_> = expr.vals.iter().map(|lit| lit.val.to_owned()).collect();
                let st = PageInfoStream::new(&pages, provider, expr.get_span());
                Ok(Self::PageInfo(st))
            },
            Expression::Link(expr) => {
                let (config, limit) = links_config_from_attributes(&expr.attributes)?;
                let st = Self::from_expr_inner(&expr.expr, provider.clone(), default_count_limit)?;
                let st = LinksStream::new(
                    Box::pin(st),
                    provider,
                    config,
                    expr.get_span(),
                );
                let st = Self::Links(st);
                // wrap around a `Counted` stream if limit is finite.
                let st = if limit.is_some_and(|x| x.is_inf()) || (limit.is_none() && default_count_limit.is_inf()) {
                    st
                } else {
                    let st = Counted::new(
                        Box::pin(st), 
                        limit.unwrap_or(default_count_limit).unwrap_int() as usize,
                        expr.get_span()
                    );
                    Self::Counted(st)
                };
                // wrap around a `Unique` stream.
                let st = Unique::new(Box::pin(st), expr.get_span());
                Ok(Self::Unique(st))
            },
            Expression::LinkTo(expr) => {
                let (config, limit) = backlinks_config_from_attributes(&expr.attributes)?;
                let st = Self::from_expr_inner(&expr.expr, provider.clone(), default_count_limit)?;
                let st = BacklinksStream::new(
                    Box::pin(st),
                    provider,
                    config,
                    expr.get_span(),
                );
                let st = Self::Backlinks(st);
                // wrap around a `Counted` stream if limit is finite.
                let st = if limit.is_some_and(|x| x.is_inf()) || (limit.is_none() && default_count_limit.is_inf()) {
                    st
                } else {
                    let st = Counted::new(
                        Box::pin(st), 
                        limit.unwrap_or(default_count_limit).unwrap_int() as usize,
                        expr.get_span()
                    );
                    Self::Counted(st)
                };
                // wrap around a `Unique` stream.
                let st = Unique::new(Box::pin(st), expr.get_span());
                Ok(Self::Unique(st))
            },
            Expression::Embed(expr) => {
                let (config, limit) = embeds_config_from_attributes(&expr.attributes)?;
                let st = Self::from_expr_inner(&expr.expr, provider.clone(), default_count_limit)?;
                let st = EmbedsStream::new(
                    Box::pin(st),
                    provider,
                    config,
                    expr.get_span(),
                );
                let st = Self::Embeds(st);
                // wrap around a `Counted` stream if limit is finite.
                let st = if limit.is_some_and(|x| x.is_inf()) || (limit.is_none() && default_count_limit.is_inf()) {
                    st
                } else {
                    let st = Counted::new(
                        Box::pin(st), 
                        limit.unwrap_or(default_count_limit).unwrap_int() as usize,
                        expr.get_span()
                    );
                    Self::Counted(st)
                };
                // wrap around a `Unique` stream.
                let st = Unique::new(Box::pin(st), expr.get_span());
                Ok(Self::Unique(st))
            },
            Expression::InCat(expr) => {
                let (config, limit, depth) = categorymembers_config_from_attributes(&expr.attributes)?;
                let st = Self::from_expr_inner(&expr.expr, provider.clone(), default_count_limit)?;
                let st = CategoryMembersStream::new(
                    Box::pin(st),
                    provider,
                    config,
                    depth.unwrap_or(IntOrInf::Int(0)),
                    expr.get_span(),
                );
                let st = Self::CategoryMembers(st);
                // wrap around a `Counted` stream if limit is finite.
                let st = if limit.is_some_and(|x| x.is_inf()) || (limit.is_none() && default_count_limit.is_inf()) {
                    st
                } else {
                    let st = Counted::new(
                        Box::pin(st), 
                        limit.unwrap_or(default_count_limit).unwrap_int() as usize,
                        expr.get_span()
                    );
                    Self::Counted(st)
                };
                // wrap around a `Unique` stream.
                let st = Unique::new(Box::pin(st), expr.get_span());
                Ok(Self::Unique(st))
            },
            Expression::Prefix(expr) => {
                let (config, limit) = prefix_config_from_attributes(&expr.attributes)?;
                let st = Self::from_expr_inner(&expr.expr, provider.clone(), default_count_limit)?;
                let st = PrefixStream::new(
                    Box::pin(st),
                    provider,
                    config,
                    expr.get_span(),
                );
                let st = Self::Prefix(st);
                // wrap around a `Counted` stream if limit is finite.
                let st = if limit.is_some_and(|x| x.is_inf()) || (limit.is_none() && default_count_limit.is_inf()) {
                    st
                } else {
                    let st = Counted::new(
                        Box::pin(st), 
                        limit.unwrap_or(default_count_limit).unwrap_int() as usize,
                        expr.get_span()
                    );
                    Self::Counted(st)
                };
                // wrap around a `Unique` stream.
                let st = Unique::new(Box::pin(st), expr.get_span());
                Ok(Self::Unique(st))
            },
            Expression::Toggle(expr) => {
                let st = Self::from_expr_inner(&expr.expr, provider, default_count_limit)?;
                let st = ToggleStream::new(Box::pin(st), expr.get_span());
                Ok(Self::Toggle(st))
            }
            _ => unimplemented!(),
        }
    }
}
