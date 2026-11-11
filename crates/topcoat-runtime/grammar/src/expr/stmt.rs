use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::Stmt;

use super::js::Js;
use crate::expr::{
    Expr,
    name_resolver::{LocalBindingKind, NameResolver},
};

impl Expr {
    /// Lowers one statement. `is_last` marks the block's trailing
    /// statement, which is its Rust value; `js_value` marks whether that
    /// value is also the JavaScript block's value. The two differ in
    /// statement position, where the Rust value is kept but JavaScript must
    /// not return it.
    pub(super) fn stmt(
        stmt: &Stmt,
        rust: &mut TokenStream,
        js: &mut Js,
        names: &mut NameResolver,
        is_last: bool,
        js_value: bool,
    ) -> syn::Result<()> {
        match stmt {
            Stmt::Local(local) => {
                let init = local.init.as_ref().ok_or_else(|| {
                    syn::Error::new_spanned(local, "let binding requires an initializer")
                })?;
                if let Some((_, diverge)) = &init.diverge {
                    return Err(syn::Error::new_spanned(
                        diverge,
                        "let-else is not supported",
                    ));
                }

                js.push_str("let ");
                let mut pat = TokenStream::new();
                let (ident, name) = Self::pat(&local.pat, &mut pat, js, names)?;
                js.push_str(" = ");
                let mut value = TokenStream::new();
                Self::dispatch(&init.expr, &mut value, js, names)?;
                js.push_str("; ");
                names.bind_local(&ident, name, LocalBindingKind::Surrogate)?;

                quote! { let #pat = #value; }.to_tokens(rust);
            }
            Stmt::Expr(expr, semi) => {
                // A trailing expression (no semicolon) is the block's Rust
                // value, so it becomes the JavaScript `return` when the
                // block's JavaScript value is used. Any other expression
                // statement discards its value: an `if` or bare block there
                // lowers as a plain JavaScript statement instead of an IIFE,
                // so `return`, `break`, and `continue` inside it reach the
                // enclosing function or loop. Note an `if` statement normally
                // carries no semicolon, so the test is whether its value is
                // used, not the presence of a semicolon.
                let returns = is_last && semi.is_none();
                let js_returns = js_value && returns;
                if js_returns {
                    js.push_str("return ");
                }

                let mut value = TokenStream::new();
                if js_returns {
                    Self::dispatch(expr, &mut value, js, names)?;
                } else {
                    match expr {
                        syn::Expr::If(inner) => Self::expr_if_stmt(inner, &mut value, js, names)?,
                        syn::Expr::Block(inner) => {
                            Self::expr_block_stmt(inner, &mut value, js, names)?;
                        }
                        _ => Self::dispatch(expr, &mut value, js, names)?,
                    }
                }

                if returns {
                    js.push(';');
                    value.to_tokens(rust);
                } else {
                    js.push_str("; ");
                    quote! { #value; }.to_tokens(rust);
                }
            }
            Stmt::Macro(stmt_macro) => Self::stmt_macro(stmt_macro, rust, js, names)?,
            other @ Stmt::Item(_) => {
                return Err(syn::Error::new_spanned(other, "unsupported statement"));
            }
        }
        Ok(())
    }
}
