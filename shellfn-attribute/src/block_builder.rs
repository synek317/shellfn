use crate::attributes::Attributes;
use crate::utils::*;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{FnArg, GenericArgument, PathArguments, ReturnType, Type, TypeImplTrait, TypeParamBound};

const PROGRAM: &'static str = "PROGRAM";

#[derive(Default)]
pub struct BlockBuilder {
    program:      String,
    cmd:          String,
    args:         Vec<String>,
    envs:         Vec<String>,
    iterator:     bool,
    outer_result: bool,
    inner_result: bool,
    no_panic:     bool,
    void:         bool,
}

impl BlockBuilder {
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

        if !self.args.iter().any(|a| a == PROGRAM) {
            self.args.push(PROGRAM.to_string());
        }

        self.no_panic = attrs.no_panic;
        self
    }

    pub fn with_args<'a>(mut self, args: impl Iterator<Item = &'a FnArg>) -> Self {
        use syn::Pat::*;
        use FnArg::*;

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
            }
            ReturnType::Type(_, ref t) => match **t {
                Type::Path(ref type_path) => {
                    if is_result_type_path(type_path) {
                        self.outer_result = true;

                        let args = &type_path.path.segments.last().unwrap().value().arguments;

                        if let PathArguments::AngleBracketed(path_args) = args {
                            if let Some(arg) = path_args.args.first() {
                                match arg.value() {
                                    GenericArgument::Type(Type::ImplTrait(ref imp)) => {
                                        self.with_impl_trait(imp)
                                    },
                                    GenericArgument::Type(Type::Tuple(ref tuple)) if tuple.elems.is_empty() => {
                                        self.void = true;
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                Type::ImplTrait(ref imp) => {
                    self.outer_result = false;
                    self.with_impl_trait(imp);
                }
                Type::Tuple(ref tuple) if tuple.elems.is_empty() => {
                    self.void = true;
                }
                ref t => panic!("Unsupported return type {:#?}", t),
            },
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
        if self.program.len() > 0 {
            self.add_program_to_args();
        } else {
            self.args.retain(|a| a != PROGRAM);
        }

        let execute_fn = self.select_execute_fn();
        let envs = self.envs;
        let cmd = self.cmd;
        let env_names = envs.iter().map(|s| s.to_uppercase()).collect::<Vec<_>>();
        let env_vals = envs
            .iter()
            .map(|e| Ident::new(&e, Span::call_site()))
            .collect::<Vec<_>>();

        // replace envs in args, e.g. for
        // #[shell(cmd = "python -m $MODULE -v"
        // fn run(module: &str)
        // it prepares following vec:
        // [
        //   "-m".to_string(),
        //   "$MODULE".replace("$MODULE", module),
        //   "-v".to_string()
        // ]
        let args = self.args
            .into_iter()
            .map(|arg|
                env_names
                    .iter()
                    .enumerate()
                    .fold(
                        quote!{ #arg },
                        |arg_tokens, (i, var_name)| {
                            let pattern = format!("${}", var_name);

                            if arg.contains(&pattern) {
                                quote!{ #arg_tokens.replace(#pattern, &envs[#i].1) }
                            } else {
                                arg_tokens
                            }
                        }
                    )
            )
            .map(|tokens| quote! { #tokens.to_string() })
            .collect::<Vec<_>>();

        // type annotation for `let envs: ...` needed because it sometimes maybe an empty vec and Command::envs is generic
        // maybe there is better way to satisfy impl IntoIterator<Item=(impl AsRef<OsStr>, impl AsRef<OsStr>)> required by envs?
        // (e.g. something that would not allocate?
        // unfortunately [("foo", bar.to_string()].into_iter() iterates over borrowed tuples, e.g. &(&str, String))
        quote! { {
            use shellfn;
            let envs: Vec<(&str, String)> = vec![#((#env_names, #env_vals.to_string())),*];
            let args: Vec<String> = vec![#(#args),*];

            #execute_fn(#cmd, args, envs)
        } }
    }

    fn add_program_to_args(&mut self) {
        for arg in self.args.iter_mut() {
            if arg == PROGRAM {
                *arg = self.program.clone()
            }
        }
    }

    fn select_execute_fn(&self) -> TokenStream2 {
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

        match (self.void, self.iterator, self.outer_result, self.inner_result, self.no_panic) {
            (VOID,   _,      NOORES, _,      NOPANIC) => quote! { shellfn::execute_void_nopanic },
            (VOID,   _,      NOORES, _,      PANIC)   => quote! { shellfn::execute_void_panic },
            (VOID,   _,      ORES,   _,      _)       => quote! { shellfn::execute_void_result },
            (NOVOID, NOITER, ORES,   _,      _)       => quote! { shellfn::execute_parse_result },
            (NOVOID, NOITER, NOORES, _,      _)       => quote! { shellfn::execute_parse_panic },
            (NOVOID, ITER,   ORES,   IRES,   _)       => quote! { shellfn::execute_iter_result_result },
            (NOVOID, ITER,   ORES,   NOIRES, NOPANIC) => quote! { shellfn::execute_iter_result_nopanic },
            (NOVOID, ITER,   ORES,   NOIRES, PANIC)   => quote! { shellfn::execute_iter_result_panic },
            (NOVOID, ITER,   NOORES, IRES,   PANIC)   => quote! { shellfn::execute_iter_panic_result },
            (NOVOID, ITER,   NOORES, IRES,   NOPANIC) => quote! { shellfn::execute_iter_nopanic_result },
            (NOVOID, ITER,   NOORES, NOIRES, NOPANIC) => quote! { shellfn::execute_iter_nopanic_nopanic },
            (NOVOID, ITER,   NOORES, NOIRES, PANIC)   => quote! { shellfn::execute_iter_panic_panic },
        }
    }
}
