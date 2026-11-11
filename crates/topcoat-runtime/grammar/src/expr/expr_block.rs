use proc_macro2::TokenStream;
use syn::ExprBlock;

use super::js::Js;
use crate::expr::{Expr, contains_await::ContainsAwait, name_resolver::NameResolver};

impl Expr {
    pub(super) fn expr_block(
        block: &ExprBlock,
        rust: &mut TokenStream,
        js: &mut Js,
        names: &mut NameResolver,
    ) -> syn::Result<()> {
        // A Rust block is already an expression; JavaScript has no block
        // expression, so it is wrapped in an immediately-invoked arrow
        // function. This expression can also hold async expressions.
        let is_async = ContainsAwait::in_block(block);
        let predicate = if is_async {
            "(await (async () => "
        } else {
            "(() => "
        };
        js.push_str(predicate);
        Self::block(&block.block, rust, js, names)?;
        js.push_str(if is_async { ")())" } else { ")()" });
        Ok(())
    }

    /// Lowers a block used as a statement, where its value is discarded.
    /// The block lowers as a plain JavaScript block instead of an IIFE, so
    /// `return`, `break`, and `continue` inside it reach the enclosing
    /// function or loop instead of exiting the generated function.
    pub(super) fn expr_block_stmt(
        block: &ExprBlock,
        rust: &mut TokenStream,
        js: &mut Js,
        names: &mut NameResolver,
    ) -> syn::Result<()> {
        Self::block_stmt(&block.block, rust, js, names)?;
        Ok(())
    }
}
