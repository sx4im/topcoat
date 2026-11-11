use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::Block;

use super::js::Js;
use crate::expr::{Expr, name_resolver::NameResolver};

impl Expr {
    /// Lowers the contents of a `{ ... }` block. The trailing expression (a
    /// statement without a semicolon) becomes the block's value.
    pub(super) fn block(
        block: &Block,
        rust: &mut TokenStream,
        js: &mut Js,
        names: &mut NameResolver,
    ) -> syn::Result<()> {
        Self::block_inner(block, rust, js, names, true)
    }

    /// Lowers the contents of a `{ ... }` block whose value is discarded
    /// (statement position). The Rust tokens are unchanged, but the trailing
    /// expression becomes an ordinary statement in JavaScript instead of the
    /// block's return, so `return`, `break`, and `continue` inside it reach
    /// the enclosing function or loop.
    pub(super) fn block_stmt(
        block: &Block,
        rust: &mut TokenStream,
        js: &mut Js,
        names: &mut NameResolver,
    ) -> syn::Result<()> {
        Self::block_inner(block, rust, js, names, false)
    }

    fn block_inner(
        block: &Block,
        rust: &mut TokenStream,
        js: &mut Js,
        names: &mut NameResolver,
        js_value: bool,
    ) -> syn::Result<()> {
        js.push_str("{ ");
        names.push_scope();

        let mut stmts = TokenStream::new();
        let last = block.stmts.len().wrapping_sub(1);
        for (i, stmt) in block.stmts.iter().enumerate() {
            Self::stmt(stmt, &mut stmts, js, names, i == last, js_value)?;
        }

        names.pop_scope();
        js.push_str(" }");
        quote! { { #stmts } }.to_tokens(rust);
        Ok(())
    }
}
