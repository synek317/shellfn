#![recursion_limit = "128"]
extern crate proc_macro;

mod attributes;
mod block_builder;
mod output_type;
mod utils;

use crate::attributes::Attributes;
use crate::block_builder::BlockBuilder;
use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, ExprLit, Lit, Stmt};

#[proc_macro_attribute]
pub fn shell(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemFn);

    let parsed_attrs = match darling::ast::NestedMeta::parse_meta_list(attr.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(darling::Error::from(e).write_errors());
        }
    };
    let attrs = match Attributes::from_list(&parsed_attrs) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    if let Some(Stmt::Expr(
        Expr::Lit(ExprLit {
            lit: Lit::Str(ref program),
            ..
        }),
        _,
    )) = input.block.stmts.first()
    {
        let mut result = input.clone();
        let program = program.value();
        let block = BlockBuilder::new()
            .with_program(program)
            .with_attrs(attrs)
            .with_args(input.sig.inputs.iter())
            .with_return_type(input.sig.output)
            .build();

        result.block = syn::parse2(block).expect("generated invalid block");

        (quote! {
            #result
        })
        .into()
    } else {
        panic!(
            r"Invalid input. Expected fn containing only string literal without any other statements"
        )
    }
}
