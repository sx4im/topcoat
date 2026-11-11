use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Expr as SynExpr, ExprIf};
use topcoat_core_grammar::common::paths::topcoat_runtime;

use super::js::Js;
use crate::expr::{Expr, contains_await::ContainsAwait, name_resolver::NameResolver};

impl Expr {
    /// Lowers `if cond { ... } else { ... }` in value position. The
    /// JavaScript side is wrapped in an IIFE because JavaScript has no `if`
    /// expression. This expression can also hold async expressions. An `if`
    /// used as a statement lowers through `expr_if_stmt` instead, without
    /// the wrapper.
    pub(super) fn expr_if(
        if_expr: &ExprIf,
        rust: &mut TokenStream,
        js: &mut Js,
        names: &mut NameResolver,
    ) -> syn::Result<()> {
        let is_async = ContainsAwait::in_if(if_expr);
        let predicate = if is_async {
            "(await (async () => "
        } else {
            "(() => "
        };
        js.push_str(predicate);
        let rust_if = Self::expr_if_inner(if_expr, js, names, false)?;
        js.push_str(if is_async { ")())" } else { ")()" });
        rust_if.to_tokens(rust);
        Ok(())
    }

    /// Lowers an `if` used as a statement, where its value is discarded.
    /// The branches lower as plain JavaScript statements instead of an IIFE,
    /// so `return`, `break`, and `continue` inside them reach the enclosing
    /// function or loop instead of exiting the generated function.
    pub(super) fn expr_if_stmt(
        if_expr: &ExprIf,
        rust: &mut TokenStream,
        js: &mut Js,
        names: &mut NameResolver,
    ) -> syn::Result<()> {
        let rust_if = Self::expr_if_inner(if_expr, js, names, true)?;
        rust_if.to_tokens(rust);
        Ok(())
    }

    fn expr_if_inner(
        if_expr: &ExprIf,
        js: &mut Js,
        names: &mut NameResolver,
        stmt_mode: bool,
    ) -> syn::Result<TokenStream> {
        js.push_str("{ if (");
        let mut cond = TokenStream::new();
        Self::dispatch(&if_expr.cond, &mut cond, js, names)?;
        js.push_str(".dehydrate()) ");

        let mut then_tokens = TokenStream::new();
        if stmt_mode {
            Self::block_stmt(&if_expr.then_branch, &mut then_tokens, js, names)?;
        } else {
            Self::block(&if_expr.then_branch, &mut then_tokens, js, names)?;
        }

        let mut else_tokens = TokenStream::new();
        let else_kw = if let Some((else_token, else_branch)) = &if_expr.else_branch {
            js.push_str(" else ");
            match &**else_branch {
                SynExpr::If(inner) => {
                    let inner_rust = Self::expr_if_inner(inner, js, names, stmt_mode)?;
                    inner_rust.to_tokens(&mut else_tokens);
                }
                SynExpr::Block(block) => {
                    if stmt_mode {
                        Self::block_stmt(&block.block, &mut else_tokens, js, names)?;
                    } else {
                        Self::block(&block.block, &mut else_tokens, js, names)?;
                    }
                }
                other => {
                    return Err(syn::Error::new_spanned(other, "unsupported else branch"));
                }
            }
            Some(else_token)
        } else {
            None
        };
        js.push_str(" }");

        let if_token = &if_expr.if_token;
        let rust = if let Some(else_kw) = else_kw {
            quote! {
                #if_token #topcoat_runtime::Surrogate::into_real(#cond)
                #then_tokens
                #else_kw #else_tokens
            }
        } else {
            quote! {
                #if_token #topcoat_runtime::Surrogate::into_real(#cond)
                #then_tokens
            }
        };
        Ok(rust)
    }
}
