#![recursion_limit="128"]
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, Ident, Span};
use quote::quote;
use syn::{Stmt, Expr, ExprLit, Lit, FnArg, ReturnType, Type, TypePath, GenericArgument, PathArguments, TypeParamBound, TypeImplTrait};
use darling::FromMeta;

fn default_cmd() -> String {
    "bash -c".to_string()
}

#[derive(Debug, Default, FromMeta)]
struct Attributes {
    #[darling(default = "default_cmd")]
    pub cmd: String,
    #[darling(default)]
    pub no_panic: bool,
}

#[derive(Default)]
struct FnBuilder {
    program:        String,
    cmd:            String,
    args:           Vec<String>,
    envs:           Vec<String>,
    iterator:       bool,
    outer_result:   bool,
    inner_result:   bool,
    no_panic:       bool,
    void:           bool,
}

// impl Default for FnBuilder {
//     fn default() -> Self {
//         Self {
//             program:        String::default(),
//             cmd:            String::default(),
//             args:           Vec::default(),
//             envs:           Vec::default(),
//             panic:          true,
//             panic_on_parse: true,
//             ignore_result:  false,
//             iterator:       false,
//         }
//     }
// }

// pub enum Error {

// }

// fn run_shell(args: , envs: ,

impl FnBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_program(mut self, program: String) -> Self {
        self.program = program;
        self
    }

    pub fn with_attrs(mut self, attrs: Attributes) -> Self {
        let mut words = shellwords::split(&attrs.cmd)
            .expect("could not parse shell command")
            .into_iter();

        self.cmd = words
            .next()
            .expect("shell command must contain at least one word");
        self.args = words.collect();

        if !self.args.iter().any(|a| a == "PROGRAM") {
            self.args.push("PROGRAM".to_string());
        }

        self.no_panic = attrs.no_panic;
        self
    }

    pub fn with_args<'a>(mut self, args: impl Iterator<Item=&'a FnArg>) -> Self {
        use FnArg::*;
        use syn::Pat::*;

        for arg in args {
            self.envs.push(
                match arg {
                    SelfRef(_) | SelfValue(_) => "self".to_string(),
                    Captured(a) => match a.pat {
                        Ident(ref pat_ident) => pat_ident.ident.to_string(),
                        Wild(_) => continue,
                        _ => panic!("captured arguments with pattern other than simple Ident are not yet supported")
                    },
                    Ignored(_) => continue,
                    Inferred(_) => panic!("inferred arguments are not yet supported")
                }
            );
        }
        self
    }

    pub fn with_return_type(mut self, return_type: ReturnType) -> Self {
        match return_type {
            ReturnType::Default => {
                self.void = true;
            },
            ReturnType::Type(_, ref t) =>
                match **t {
                    Type::Path(ref type_path) => {
                        if is_result_type_path(type_path) {
                            self.outer_result = true;

                            let args = &type_path.path.segments.last().unwrap().value().arguments;

                            if let PathArguments::AngleBracketed(path_args) = args {
                                if let Some(arg) = path_args.args.first() {
                                    if let GenericArgument::Type(Type::ImplTrait(ref imp)) = arg.value() {
                                        self.with_impl_trait(imp)
                                    }
                                }
                            }
                        }
                    },
                    Type::ImplTrait(ref imp) => {
                        self.outer_result = false;
                        self.with_impl_trait(imp);
                    },
                    _ => {}
                }
        }
        self
    }

    fn with_impl_trait(&mut self, imp: &TypeImplTrait) {
        if let Some(t) = imp.bounds.first() {
            if let TypeParamBound::Trait(ref bound) = t.value() {
                if let Some(pair) = bound.path.segments.first() {
                    let segment = pair.value();

                    if segment.ident.to_string() == "Iterator" {
                        self.iterator = true;

                        if let PathArguments::AngleBracketed(ref path_args) = segment.arguments {
                            if let Some(arg) = path_args.args.first() {
                                if let GenericArgument::Binding(ref binding) = arg.value() {
                                    if binding.ident.to_string() == "Item" {
                                        if is_result_type(&binding.ty) {
                                            self.inner_result = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn build(mut self) -> TokenStream2 {
        self.add_program_to_args();

        let cmd  = self.cmd;
        let args = self.args;
        let env_names = self.envs
            .iter()
            .map(|s| s.to_uppercase());
        let env_vals = self.envs
            .iter()
            .map(|e| Ident::new(&e, Span::call_site()));

        let mut result = quote! {
            use shellfn;
            // type annotation needed because it sometimes maybe an empty vec and Command::envs is generic
            // maybe there is better way to satisfy impl IntoIterator<Item=(impl AsRef<OsStr>, impl AsRef<OsStr>)> required by envs?
            // (e.g. something that would not allocate?
            // unfortunately [("foo", bar.to_string()].into_iter() iterates over borrowed tuples, e.g. &(&str, String))
            let envs: Vec<(&str, String)> = vec![#((#env_names, #env_vals.to_string())),*];
            let args = vec![#(#args),*];
        };

        const VOID:    bool = true;
        const NOVOID:  bool = false;
        const ITER:    bool = true;
        const NOITER:  bool = false;
        const ORES:    bool = true;
        const NOORES:  bool = false;
        const IRES:    bool = true;
        const NOIRES:  bool = false;
        const NOPANIC: bool = true;
        const PANIC:   bool = false;

        result.extend(match (self.void, self.iterator, self.outer_result, self.inner_result, self.no_panic) {
            (VOID,   _,      _,      _,      NOPANIC) => quote! { shellfn::execute_void_nopanic(#cmd, args, envs); },
            (VOID,   _,      _,      _,      PANIC)   => quote! { shellfn::execute_void_panic(#cmd, args, envs); },
            (NOVOID, NOITER, ORES,   _,      _)       => quote! { shellfn::execute_parse_result(#cmd, args, envs) },
            (NOVOID, NOITER, NOORES, _,      _)       => quote! { shellfn::execute_parse_panic(#cmd, args, envs) },
            (NOVOID, ITER,   ORES,   IRES,   _)       => quote! { shellfn::execute_iter_result_result(#cmd, args, envs) },
            (NOVOID, ITER,   ORES,   NOIRES, NOPANIC) => quote! { shellfn::execute_iter_result_nopanic(#cmd, args, envs) },
            (NOVOID, ITER,   ORES,   NOIRES, PANIC)   => quote! { shellfn::execute_iter_result_panic(#cmd, args, envs) },
            (NOVOID, ITER,   NOORES, IRES,   PANIC)   => quote! { shellfn::execute_iter_panic_result(#cmd, args, envs) },
            (NOVOID, ITER,   NOORES, IRES,   NOPANIC) => quote! { shellfn::execute_iter_nopanic_result(#cmd, args, envs) },
            (NOVOID, ITER,   NOORES, NOIRES, NOPANIC) => quote! { shellfn::execute_iter_nopanic_nopanic(#cmd, args, envs) },
            (NOVOID, ITER,   NOORES, NOIRES, PANIC)   => quote! { shellfn::execute_iter_panic_panic(#cmd, args, envs) },
        });

        quote! { { #result } }
    }

    fn add_program_to_args(&mut self) {
        for arg in self.args.iter_mut() {
            if arg == "PROGRAM" {
                *arg = self.program.clone()
            }
        }
    }
}

fn is_result_type(typ: &Type) -> bool {
    if let Type::Path(ref type_path) = *typ {
        is_result_type_path(type_path)
    }
    else {
        false
    }
}

fn is_result_type_path(type_path: &TypePath) -> bool {
    type_path
        .path
        .segments
        .last()
        .map_or(false, |s| s.value().ident.to_string() == "Result")
}

    // fn get_path(typ: Box<Type>) -> Option<Path> {
    //     use syn::Type::*;

    //     match typ {
    //         Slice(s) => panic!("S
    //     }
    // }

#[proc_macro_attribute]
pub fn shell(attr: TokenStream, input: TokenStream) -> TokenStream {
    //println!("ATTR:\n{:#?}", attr);
    //println!("--------");
    //println!("INPUT:\n{:#?}", input);
    let input = syn::parse_macro_input!(input as syn::ItemFn);
    //println!("PARSED INPUT:\n{:#?}", input);
    let attr = syn::parse_macro_input!(attr as syn::AttributeArgs);

    // // workaround for darling, it is transforming
    // // #[shell(cmd = "...")]
    // // into
    // // #[shell(dummy(cmd = "..."))]
    // // Is there any better solution?
    // let attr = syn::Meta::List(
    //     syn::MetaList {
    //         ident: syn::Ident::new("dummy", proc_macro2::Span::call_site()),
    //         paren_token: syn::token::Paren { span: proc_macro2::Span::call_site() },
    //         nested: {
    //             let mut result = syn::punctuated::Punctuated::new();
    //             for nested in attr {
    //                 result.push(nested);
    //             }
    //             result
    //         }
    //     }
    // );

    //println!("PARSED ATTR:\n{:#?}", attr);
    let attrs = Attributes::from_list(&attr).expect("Meta");
    //println!("ATTRS:\n{:#?}", attrs);
    if let Some(Stmt::Expr(Expr::Lit(ExprLit { lit: Lit::Str(ref program), .. }))) = input.block.stmts.iter().next() {
        let mut result = input.clone();

        //let tokens = if
        let program = program.value();
        ////println!("AAA: {}", spawn_process(&opts.cmd, &program).to_string());
        let block = FnBuilder::new()
                .with_program(program)
                .with_attrs(attrs)
                .with_args(input.decl.inputs.iter())
                .with_return_type(input.decl.output)
                .build();

        //println!("BLOCK:\n{}", block.to_string());

        result.block = syn::parse2(block).expect("generated invalid block");
        // result.block = syn::parse2(quote!{{
        //     use std::process::{Command, Stdio};
        //     use std::sync::mpsc;
        //     use std::thread;
        //     use std::io::BufReader;
        //     //"baz".to_string()
        //     let mut process = Command::new("bash")
        //         //.stdin(Stdio::piped())
        //         .stdout(Stdio::piped())
        //         //.stderr(Stdio::inherit())
        //         .args(&["-c", #program])
        //         //.arg("-c")
        //         //.arg("bazbaz")
        //         .spawn();

        //     let process = process.unwrap();
        //     let output = process.wait_with_output();
        //     let output = output.unwrap();

        //     // let mut s = String::new();
        //     // use std::io::Read;
        //     // process.stdout.unwrap().read_to_string(&mut s);
        //     String::from_utf8(output.stdout).unwrap()
        //     //String::from_utf8(output.stdout).unwrap()
        //     // "baz".to_string()
        //     // let (tx, rx) = mpsc::channel();
        //     // let stdout = process.stdout.take().unwrap();

        //     // thread::spawn(move || {
        //     //     let reader = BufReader::new(stdout);

        //     //     for line in reader.lines() {
        //     //         tx.send(Some(line.unwrap()));
        //     //     }
        //     // });

        //     // loop {
        //     //     let data = process.rx.try_recv();
        //     //     if data.is_ok() {
        //     //         let data = data.unwrap();
        //     //         //println!("{:?}", data);
        //     //     }
        //     // }
        // }}).unwrap();

        (quote! {
            #result
        }).into()
    }
    else {
        panic!(r"Invalid input. Expected fn containing only string literal without any other statements")
    }
}
