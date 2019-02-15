#![recursion_limit = "128"]
extern crate proc_macro;

mod attributes;
mod block_builder;
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
    let attr_args = syn::parse_macro_input!(attr as syn::AttributeArgs);
    let attrs = Attributes::from_list(&attr_args).expect("Meta");

    if let Some(Stmt::Expr(Expr::Lit(ExprLit {
        lit: Lit::Str(ref program),
        ..
    }))) = input.block.stmts.iter().next()
    {
        let mut result = input.clone();
        let program = program.value();
        let block = BlockBuilder::new()
            .with_program(program)
            .with_attrs(attrs)
            .with_args(input.decl.inputs.iter())
            .with_return_type(input.decl.output)
            .build();

        result.block = syn::parse2(block).expect("generated invalid block");

        (quote! {
            #result
        })
        .into()
    } else {
        panic!(r"Invalid input. Expected fn containing only string literal without any other statements")
    }
}
