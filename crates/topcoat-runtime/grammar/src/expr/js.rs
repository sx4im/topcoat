use std::{fmt, ops::AddAssign};

use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote};
use topcoat_core_grammar::common::paths::topcoat_runtime;

/// JavaScript source interleaved with expressions captured by the server.
#[derive(Default)]
pub(super) struct Js {
    source: String,
    parts: TokenStream,
    /// How many generated immediately-invoked functions enclose the current
    /// position. A `return`, `break`, or `continue` inside one of those
    /// exits the generated function instead of the enclosing closure or loop.
    synthetic: u32,
}

impl Js {
    pub(super) fn push_str(&mut self, source: &str) {
        self.source.push_str(source);
    }

    pub(super) fn push(&mut self, character: char) {
        self.source.push(character);
    }

    /// Records entry into a generated immediately-invoked function.
    pub(super) fn enter_synthetic(&mut self) {
        self.synthetic += 1;
    }

    /// Records exit from a generated immediately-invoked function.
    pub(super) fn exit_synthetic(&mut self) {
        self.synthetic -= 1;
    }

    /// Whether the current position is inside a generated immediately-invoked
    /// function.
    #[must_use]
    pub(super) fn in_synthetic(&self) -> bool {
        self.synthetic > 0
    }

    /// Records entry into a real closure body, which is a fresh function
    /// boundary. Returns the previous depth so it can be restored.
    #[must_use]
    pub(super) fn enter_function(&mut self) -> u32 {
        std::mem::replace(&mut self.synthetic, 0)
    }

    /// Restores the depth saved by [`Js::enter_function`].
    pub(super) fn exit_function(&mut self, saved: u32) {
        self.synthetic = saved;
    }

    pub(super) fn expression(&mut self, ident: &Ident) {
        let source = std::mem::take(&mut self.source);
        quote! { .source(#source).expression(&#ident) }.to_tokens(&mut self.parts);
    }
}

impl ToTokens for Js {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let parts = &self.parts;
        let source = &self.source;
        quote! { #topcoat_runtime::Js::builder() #parts .source(#source).build() }
            .to_tokens(tokens);
    }
}

impl fmt::Write for Js {
    fn write_str(&mut self, source: &str) -> fmt::Result {
        self.push_str(source);
        Ok(())
    }
}

impl AddAssign<&str> for Js {
    fn add_assign(&mut self, source: &str) {
        self.push_str(source);
    }
}
