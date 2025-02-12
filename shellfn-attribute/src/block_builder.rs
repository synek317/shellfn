use crate::attributes::Attributes;
use crate::output_type::OutputType;
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
    output_type:  OutputType,
    outer_result: bool,
    inner_result: bool,
    no_panic:     bool,
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
                self.with_unit_return_type();
            }
            ReturnType::Type(_, ref t) => match **t {
                Type::Path(ref type_path) if is_result_type_path(type_path) => {
                    self.outer_result = true;

                    let args = &type_path.path.segments.last().unwrap().value().arguments;

                    if let PathArguments::AngleBracketed(path_args) = args {
                        if let Some(arg) = path_args.args.first() {
                            match arg.value() {
                                GenericArgument::Type(Type::ImplTrait(ref imp)) => {
                                    self.with_impl_trait(imp)
                                }
                                GenericArgument::Type(ref t) if is_unit_type(t) => {
                                    self.with_unit_return_type();
                                }
                                GenericArgument::Type(ref t) if is_vec_type(t) => {
                                    self.with_vec_return_type(t);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Type::ImplTrait(ref imp) => {
                    self.outer_result = false;
                    self.with_impl_trait(imp);
                }
                ref t if is_vec_type(t) => self.with_vec_return_type(t),
                ref t if is_unit_type(t) => self.with_unit_return_type(),
                Type::Path(_) => {}
                ref t => panic!("Unsupported return type {:#?}", t),
            },
        }
        self
    }

    fn with_unit_return_type(&mut self) {
        self.output_type = OutputType::Void;
    }

    fn with_vec_return_type(&mut self, typ: &Type) {
        self.output_type = OutputType::Vec;

        if let Type::Path(ref type_path) = typ {
            let args = &type_path.path.segments.last().unwrap().value().arguments;

            if let PathArguments::AngleBracketed(path_args) = args {
                if let Some(arg) = path_args.args.first() {
                    if let GenericArgument::Type(ref t) = arg.value() {
                        self.inner_result = is_result_type(t);
                    }
                }
            }
        }
    }

    fn with_impl_trait(&mut self, imp: &TypeImplTrait) {
        if let Some(t) = imp.bounds.first() {
            if let TypeParamBound::Trait(ref bound) = t.value() {
                if let Some(pair) = bound.path.segments.first() {
                    let segment = pair.value();

                    if segment.ident.to_string() == "Iterator" {
                        self.output_type = OutputType::Iter;

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
            .map(|e| Ident::new(e, Span::call_site()))
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
        let args = self
            .args
            .into_iter()
            .map(|arg|
                env_names
                    .iter()
                    .enumerate()
                    .fold(quote! { #arg }, |arg_tokens, (i, var_name)| {
                        if arg == PROGRAM {
                            return arg_tokens;
                        }

                        let pattern = format!("${}", var_name);

                        if arg.contains(&pattern) {
                            quote! { #arg_tokens.replace(#pattern, &envs[#i].1) }
                        } else {
                            arg_tokens
                        }
                    })
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
        use OutputType::*;

        const ORES:    bool = true; // outer result, like Result<impl Iterator<Item=T>, E>
        const NOORES:  bool = false;
        const IRES:    bool = true; // inner result, like impl Iterator<Item=Result<T, E>>
        const NOIRES:  bool = false;
        const NOPANIC: bool = true;
        const PANIC:   bool = false;

        match (
            &self.output_type,
            self.outer_result,
            self.inner_result,
            self.no_panic,
        ) {
            (Void, NOORES, _,      NOPANIC) => quote! { shellfn::execute_void_nopanic },
            (Void, NOORES, _,      PANIC)   => quote! { shellfn::execute_void_panic },
            (Void, ORES,   _,      _)       => quote! { shellfn::execute_void_result },
            (T,    ORES,   _,      _)       => quote! { shellfn::execute_parse_result },
            (T,    NOORES, _,      _)       => quote! { shellfn::execute_parse_panic },
            (Iter, ORES,   IRES,   _)       => quote! { shellfn::execute_iter_result_result },
            (Iter, ORES,   NOIRES, NOPANIC) => quote! { shellfn::execute_iter_result_nopanic },
            (Iter, ORES,   NOIRES, PANIC)   => quote! { shellfn::execute_iter_result_panic },
            (Iter, NOORES, IRES,   PANIC)   => quote! { shellfn::execute_iter_panic_result },
            (Iter, NOORES, IRES,   NOPANIC) => quote! { shellfn::execute_iter_nopanic_result },
            (Iter, NOORES, NOIRES, NOPANIC) => quote! { shellfn::execute_iter_nopanic_nopanic },
            (Iter, NOORES, NOIRES, PANIC)   => quote! { shellfn::execute_iter_panic_panic },
            (Vec,  ORES,   IRES,   _)       => quote! { shellfn::execute_vec_result_result },
            (Vec,  ORES,   NOIRES, NOPANIC) => quote! { shellfn::execute_vec_result_nopanic },
            (Vec,  ORES,   NOIRES, PANIC)   => quote! { shellfn::execute_vec_result_panic },
            (Vec,  NOORES, IRES,   PANIC)   => quote! { shellfn::execute_vec_panic_result },
            (Vec,  NOORES, IRES,   NOPANIC) => quote! { shellfn::execute_vec_nopanic_result },
            (Vec,  NOORES, NOIRES, NOPANIC) => quote! { shellfn::execute_vec_nopanic_nopanic },
            (Vec,  NOORES, NOIRES, PANIC)   => quote! { shellfn::execute_vec_panic_panic },
        }
    }
}
