use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Expr as SynExpr, ExprIf};
use topcoat_core_grammar::common::paths::topcoat_runtime;

use super::js::Js;
use crate::expr::{Expr, contains_await::ContainsAwait, name_resolver::NameResolver};

impl Expr {
    /// Lowers `if cond { ... } else { ... }`. In expression position the
    /// JavaScript side is wrapped in an IIFE so the same shape works as a
    /// value. This expression can also hold async expressions.
    ///
    /// When `statement` is true, the conditional is used as a statement, so
    /// it is emitted directly without the wrapper and its branches are
    /// lowered as statements. That keeps `return`, `break`, and `continue`
    /// inside the branches bound to the enclosing closure or loop.
    pub(super) fn expr_if(
        if_expr: &ExprIf,
        rust: &mut TokenStream,
        js: &mut Js,
        names: &mut NameResolver,
        statement: bool,
    ) -> syn::Result<()> {
        if statement {
            let rust_if = Self::expr_if_inner(if_expr, js, names, false)?;
            rust_if.to_tokens(rust);
            return Ok(());
        }
        let is_async = ContainsAwait::in_if(if_expr);
        let predicate = if is_async {
            "(await (async () => "
        } else {
            "(() => "
        };
        js.push_str(predicate);
        js.enter_synthetic();
        let inner = Self::expr_if_inner(if_expr, js, names, true);
        js.exit_synthetic();
        let rust_if = inner?;
        js.push_str(if is_async { ")())" } else { ")()" });
        rust_if.to_tokens(rust);
        Ok(())
    }

    fn expr_if_inner(
        if_expr: &ExprIf,
        js: &mut Js,
        names: &mut NameResolver,
        value: bool,
    ) -> syn::Result<TokenStream> {
        js.push_str("{ if (");
        let mut cond = TokenStream::new();
        Self::dispatch(&if_expr.cond, &mut cond, js, names)?;
        js.push_str(".dehydrate()) ");

        let mut then_tokens = TokenStream::new();
        Self::block(&if_expr.then_branch, &mut then_tokens, js, names, value)?;

        let mut else_tokens = TokenStream::new();
        let else_kw = if let Some((else_token, else_branch)) = &if_expr.else_branch {
            js.push_str(" else ");
            match &**else_branch {
                SynExpr::If(inner) => {
                    let inner_rust = Self::expr_if_inner(inner, js, names, value)?;
                    inner_rust.to_tokens(&mut else_tokens);
                }
                SynExpr::Block(block) => {
                    Self::block(&block.block, &mut else_tokens, js, names, value)?;
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
