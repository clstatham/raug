use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse::Parser, parse_macro_input, punctuated::Punctuated};

struct ProcessorArg {
    name: syn::Ident,
    ty: syn::Type,
}

impl ToTokens for ProcessorArg {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ProcessorArg { name, ty, .. } = self;
        tokens.extend(quote! {
            #name: #ty,
        });
    }
}

pub fn processor_attribute(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated.parse(attr);

    let mut extra_derives = vec![];
    let mut allocate_fn = None;
    let mut resize_buffers_fn = None;
    if let Ok(args) = args {
        for arg in args.iter() {
            if let syn::Meta::List(meta_list) = arg {
                if meta_list.path.is_ident("derive") {
                    meta_list
                        .parse_nested_meta(|meta| {
                            let ident = meta.path.get_ident().unwrap();
                            extra_derives.push(ident.clone());
                            Ok(())
                        })
                        .unwrap();
                } else {
                    return syn::Error::new_spanned(
                        meta_list.path.clone(),
                        "Unknown attribute. Only `derive` is supported.",
                    )
                    .to_compile_error()
                    .into();
                }
            } else if let syn::Meta::NameValue(meta_name_value) = arg {
                if meta_name_value.path.is_ident("allocate") {
                    if let syn::Expr::Path(path) = &meta_name_value.value {
                        allocate_fn = Some(path.path.clone());
                    } else {
                        return syn::Error::new_spanned(
                            meta_name_value.value.clone(),
                            "Expected a path for `allocate_fn`",
                        )
                        .to_compile_error()
                        .into();
                    }
                } else if meta_name_value.path.is_ident("resize_buffers") {
                    if let syn::Expr::Path(path) = &meta_name_value.value {
                        resize_buffers_fn = Some(path.path.clone());
                    } else {
                        return syn::Error::new_spanned(
                            meta_name_value.value.clone(),
                            "Expected a path for `resize_buffers_fn`",
                        )
                        .to_compile_error()
                        .into();
                    }
                } else {
                    return syn::Error::new_spanned(
                        meta_name_value.path.clone(),
                        "Unknown attribute. Only `allocate_fn` and `resize_buffers_fn` are supported.",
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }
    }

    let item = parse_macro_input!(item as syn::ItemFn);
    let vis = item.vis.clone();
    let (ig, tg, wc) = item.sig.generics.split_for_impl();
    let func_name = item.sig.ident.clone().to_string();
    let struct_name = func_name.to_case(Case::Pascal);
    let struct_name = format_ident!("{}", struct_name);
    let attrs = item.attrs.clone();

    let mut proc_env_ident = None;

    let mut phantom_data = vec![];
    let mut state = vec![];
    let mut input = vec![];
    let mut output = vec![];
    let mut input_spec = vec![];
    let mut output_spec = vec![];
    let mut create_output_buffers = vec![];
    let mut update_args = vec![];
    let mut update_call_args = vec![];
    let mut get_inputs = vec![];
    let mut get_outputs = vec![];
    let mut assign_inputs = vec![];
    let mut assign_outputs = vec![];

    for generic in item.sig.generics.params.iter() {
        if let syn::GenericParam::Type(ty) = generic {
            let ty = &ty.ident;
            let ident = format_ident!("_{}", ty.to_string().to_lowercase());

            phantom_data.push(quote! {
                #ident: std::marker::PhantomData<#ty>,
            });
        }
    }

    for arg in item.sig.inputs.iter() {
        if let syn::FnArg::Typed(arg) = arg {
            if let syn::Type::Path(type_path) = &*arg.ty {
                if type_path.path.is_ident("ProcEnv") {
                    if proc_env_ident.is_some() {
                        return syn::Error::new_spanned(
                            arg.pat.clone(),
                            "Only one ProcEnv argument is allowed",
                        )
                        .to_compile_error()
                        .into();
                    }
                    proc_env_ident = Some(arg.pat.clone());
                    continue;
                }
            }
            if let Some(attr) = arg.attrs.first() {
                if attr.path().is_ident("state") {
                    let name = if let syn::Pat::Ident(pat_ident) = &*arg.pat {
                        pat_ident.ident.clone()
                    } else {
                        return syn::Error::new_spanned(
                            arg.pat.clone(),
                            "State argument must be a named identifier",
                        )
                        .to_compile_error()
                        .into();
                    };
                    let ty = *arg.ty.clone();
                    state.push(ProcessorArg { name, ty });
                } else if attr.path().is_ident("input") {
                    let name = if let syn::Pat::Ident(pat_ident) = &*arg.pat {
                        pat_ident.ident.clone()
                    } else {
                        return syn::Error::new_spanned(
                            arg.pat.clone(),
                            "Input argument must be a named identifier",
                        )
                        .to_compile_error()
                        .into();
                    };
                    let ty;
                    if let syn::Type::Group(group) = &*arg.ty {
                        if let syn::Type::Reference(reference) = &*group.elem {
                            if reference.mutability.is_some() {
                                return syn::Error::new_spanned(
                                    arg.pat.clone(),
                                    "Input argument must be immutable reference",
                                )
                                .to_compile_error()
                                .into();
                            }
                            ty = *reference.elem.clone();
                        } else {
                            return syn::Error::new_spanned(
                                arg.pat.clone(),
                                "Input argument must be a reference inside a group",
                            )
                            .to_compile_error()
                            .into();
                        }
                    } else if let syn::Type::Reference(reference) = &*arg.ty {
                        if reference.mutability.is_some() {
                            return syn::Error::new_spanned(
                                arg.pat.clone(),
                                "Input argument must be immutable reference",
                            )
                            .to_compile_error()
                            .into();
                        }
                        ty = *reference.elem.clone();
                    } else {
                        return syn::Error::new_spanned(
                            arg.pat.clone(),
                            "Input argument must be a reference",
                        )
                        .to_compile_error()
                        .into();
                    }
                    input.push(ProcessorArg { name, ty });
                } else if attr.path().is_ident("output") {
                    let name = if let syn::Pat::Ident(pat_ident) = &*arg.pat {
                        pat_ident.ident.clone()
                    } else {
                        return syn::Error::new_spanned(
                            arg.pat.clone(),
                            "Output argument must be a named identifier",
                        )
                        .to_compile_error()
                        .into();
                    };
                    let ty;
                    if let syn::Type::Group(group) = &*arg.ty {
                        if let syn::Type::Reference(reference) = &*group.elem {
                            if reference.mutability.is_none() {
                                return syn::Error::new_spanned(
                                    arg.pat.clone(),
                                    "Output argument must be mutable reference",
                                )
                                .to_compile_error()
                                .into();
                            }
                            ty = *reference.elem.clone();
                        } else {
                            return syn::Error::new_spanned(
                                arg.pat.clone(),
                                "Output argument must be a reference inside a group",
                            )
                            .to_compile_error()
                            .into();
                        }
                    } else if let syn::Type::Reference(reference) = &*arg.ty {
                        if reference.mutability.is_none() {
                            return syn::Error::new_spanned(
                                arg.pat.clone(),
                                "Output argument must be mutable reference",
                            )
                            .to_compile_error()
                            .into();
                        }
                        ty = *reference.elem.clone();
                    } else {
                        return syn::Error::new_spanned(
                            arg.pat.clone(),
                            "Output argument must be a reference",
                        )
                        .to_compile_error()
                        .into();
                    }
                    output.push(ProcessorArg { name, ty });
                } else {
                    return syn::Error::new_spanned(
                        attr.path().clone(),
                        "Unknown attribute. Only `state`, `input`, and `output` are supported.",
                    )
                    .to_compile_error()
                    .into();
                }
            } else {
                return syn::Error::new_spanned(
                    arg.pat.clone(),
                    "Expected a function argument with attributes",
                )
                .to_compile_error()
                .into();
            }
        } else {
            return syn::Error::new_spanned(
                arg.clone(),
                "Expected a function argument with attributes",
            )
            .to_compile_error()
            .into();
        }
    }

    let proc_env_decl = if let Some(proc_env_ident) = proc_env_ident {
        quote! {
            let #proc_env_ident = env;
        }
    } else {
        quote! {}
    };

    let mut struct_fields = vec![];
    for arg in state.iter() {
        let ProcessorArg { name, ty, .. } = arg;
        let ty = if let syn::Type::Reference(ty) = ty {
            if ty.mutability.is_none() {
                return syn::Error::new_spanned(
                    ty.clone(),
                    "State argument must be mutable reference",
                )
                .to_compile_error()
                .into();
            }
            ty.elem.clone()
        } else {
            return syn::Error::new_spanned(
                ty.clone(),
                "State argument must be a mutable reference",
            )
            .to_compile_error()
            .into();
        };
        struct_fields.push(quote! {
            pub #name: #ty,
        });
        update_args.push(quote! {
            #name: &mut #ty,
        });
        update_call_args.push(quote! {
            &mut self.#name,
        })
    }

    for (arg_index, arg) in input.iter().enumerate() {
        let ProcessorArg { name, ty } = arg;

        struct_fields.push(quote! {
            pub #name: #ty,
        });
        input_spec.push(quote! {
            raug::processor::io::SignalSpec::new(stringify!(#name), <#ty as raug::signal::Signal>::signal_type())
        });
        get_inputs.push(quote! {
            let #name = &inputs.input_as::<#ty>(#arg_index);
        });
        assign_inputs.push(quote! {
            if let Some(#name) = #name.map(|inp| &inp[__i]) {
                Clone::clone_from(&mut self.#name, #name);
            }
        });
        update_args.push(quote! {
            #name: &#ty,
        });
        update_call_args.push(quote! {
            &self.#name,
        });
    }

    for (arg_index, arg) in output.iter().enumerate() {
        let ProcessorArg { name, ty } = arg;

        output_spec.push(quote! {
            raug::processor::io::SignalSpec::new(stringify!(#name), <#ty as raug::signal::Signal>::signal_type())
        });
        create_output_buffers.push(quote! {
            raug::signal::type_erased::AnyBuffer::zeros::<#ty>(size)
        });
        update_args.push(quote! {
            #name: &mut #ty,
        });
        update_call_args.push(quote! {
            #name,
        });
        get_outputs.push(quote! {
            // SAFETY: We won't ever get the same output buffer twice, so there's no way to alias it.
            let mut #name = unsafe { outputs.output_extended_lifetime(#arg_index) };
        });
        assign_outputs.push(quote! {
            let #name = #name.get_mut_as::<#ty>(__i).unwrap();
        });
    }

    let struct_def = quote! {
        #(#attrs)*
        #[derive(#(#extra_derives),*)]
        #[allow(missing_docs)]
        #vis struct #struct_name #tg #wc {
            #(#struct_fields)*
            #(#phantom_data)*
        }
    };

    let body = item.block.clone();

    let struct_update_impl = quote! {
        impl #ig #struct_name #tg #wc {
            #[doc = "Update function for the processor."]
            #(#attrs)*
            #[allow(clippy::too_many_arguments)]
            #[allow(clippy::ptr_arg)]
            #[track_caller]
            pub fn process_sample(env: raug::processor::io::ProcEnv, #(#update_args)*) -> raug::processor::ProcResult<()> {
                #proc_env_decl
                #body
            }
        }
    };

    let mut node_inputs = vec![];
    let mut node_fn_args = vec![];

    for input in input.iter() {
        let ProcessorArg { name, .. } = input;
        node_inputs.push(quote! {
            #name
        });
        node_fn_args.push(quote! {
            #name: impl raug::graph::node::IntoOutputOpt,
        });
    }

    let node_fn_def = quote! {
        impl #ig #struct_name #tg #wc {
            #[doc = concat!("Adds a new ", stringify!(#struct_name), "node to the graph and connects its inputs.")]
            #[allow(unused)]
            #[allow(clippy::too_many_arguments)]
            #[track_caller]
            #vis fn node(self, graph: &raug::graph::Graph, #(#node_fn_args)*) -> raug::graph::node::Node {
                use raug::graph::node::IntoOutputOpt;
                let node = graph.node(self);
                let mut input_index = 0;
                #(
                    if let Some(input) = #node_inputs.into_output_opt(graph) {
                        node.input(input_index).connect(input);
                    }
                    input_index += 1;
                )*
                node
            }
        }
    };

    // let outputs = item.sig.output.clone();

    let allocate_fn = if let Some(allocate_fn) = allocate_fn {
        quote! {
            fn allocate(&mut self, sample_rate: f32, block_size: usize) {
                #allocate_fn(self, sample_rate, block_size);
            }
        }
    } else {
        quote! {}
    };

    let resize_buffers_fn = if let Some(resize_buffers_fn) = resize_buffers_fn {
        quote! {
            fn resize_buffers(&mut self, sample_rate: f32, block_size: usize) {
                #resize_buffers_fn(self, sample_rate, block_size);
            }
        }
    } else {
        quote! {}
    };

    let processor_impl = quote! {
        impl #ig raug::processor::Processor for #struct_name #tg #wc {
            fn name(&self) -> &str {
                stringify!(#struct_name)
            }

            fn input_spec(&self) -> Vec<raug::processor::io::SignalSpec> {
                vec![#(#input_spec),*]
            }

            fn output_spec(&self) -> Vec<raug::processor::io::SignalSpec> {
                vec![#(#output_spec),*]
            }

            fn create_output_buffers(&self, size: usize) -> Vec<raug::signal::type_erased::AnyBuffer> {
                vec![#(#create_output_buffers),*]
            }

            #allocate_fn
            #resize_buffers_fn

            #[track_caller]
            fn process(&mut self, inputs: raug::processor::io::ProcessorInputs, mut outputs: raug::processor::io::ProcessorOutputs) -> Result<(), raug::processor::ProcessorError> {
                #(#get_inputs)*
                #(#get_outputs)*

                for __i in 0..inputs.block_size() {
                    #(#assign_inputs)*
                    #(#assign_outputs)*
                    Self::process_sample(inputs.env, #(#update_call_args)*)?;
                }

                Ok(())
            }
        }
    };

    quote! {
        #struct_def
        #struct_update_impl
        #node_fn_def
        #processor_impl
    }
    .into()
}
