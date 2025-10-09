use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;

struct InnerModule {
    module: syn::Ident,
    procs: Punctuated<syn::Type, syn::Token![,]>,
}

impl syn::parse::Parse for InnerModule {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<syn::Token![mod]>()?;
        let module = input.parse()?;
        let content;
        syn::braced!(content in input);
        let procs = content.parse_terminated(syn::Type::parse, syn::Token![,])?;
        Ok(Self { module, procs })
    }
}

struct WasmInput {
    inner_modules: Vec<InnerModule>,
}

impl syn::parse::Parse for WasmInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut inner_modules = vec![];
        while !input.is_empty() {
            let inner_module: InnerModule = input.parse()?;
            inner_modules.push(inner_module);
        }
        Ok(Self { inner_modules })
    }
}

pub fn wrap_wasm(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as WasmInput);

    let mut outputs = vec![];

    for inner_module in input.inner_modules {
        let module = inner_module.module;
        let procs = inner_module.procs;

        let mut output = quote! {};

        for proc in &procs {
            let (func_name, type_name) = match proc {
                syn::Type::Path(path) => {
                    // Get the last segment of the path
                    let segment = path.path.segments.last().unwrap();
                    // if there's a generic argument, include it in the type name
                    let type_name = if let syn::PathArguments::AngleBracketed(args) =
                        &segment.arguments
                    {
                        if args.args.len() == 1 {
                            if let syn::GenericArgument::Type(syn::Type::Path(arg_path)) =
                                &args.args[0]
                            {
                                let segment_ident = segment.ident.to_string().to_case(Case::Pascal);
                                let arg_segment = arg_path.path.segments.last().unwrap();
                                let arg_ident =
                                    &arg_segment.ident.to_string().to_case(Case::Pascal);
                                let arg_ident = match arg_ident.as_str() {
                                    "F32" => "Float",
                                    "F64" => "Double",
                                    "i32" => "Int",
                                    "i64" => "Long",
                                    "U32" => "Uint",
                                    "U64" => "Ulong",
                                    "Bool" => "Bool",
                                    other => other,
                                };
                                format_ident!("{}{}", segment_ident, arg_ident)
                            } else {
                                panic!("Expected type argument");
                            }
                        } else {
                            panic!("Expected one generic argument");
                        }
                    } else {
                        segment.ident.clone()
                    };
                    (
                        format_ident!("{}", type_name.to_string().to_case(Case::Camel)),
                        type_name,
                    )
                }
                _ => panic!("Expected type path"),
            };
            let wrapped = quote! {
                #[wasm_bindgen]
                pub struct #type_name {
                    pub(crate) inner: #module::builtins::#proc,
                }

                #[wasm_bindgen]
                impl #type_name {
                    #[wasm_bindgen(constructor)]
                    pub fn new() -> Self {
                        Self {
                            inner: <#module::builtins::#proc as Default>::default(),
                        }
                    }

                    #[wasm_bindgen(js_name = "name")]
                    pub fn name(&self) -> String {
                        self.inner.name().to_string()
                    }

                    #[wasm_bindgen(js_name = "numInputs")]
                    pub fn num_inputs(&self) -> u32 {
                        self.inner.input_spec().len() as u32
                    }

                    #[wasm_bindgen(js_name = "numOutputs")]
                    pub fn num_outputs(&self) -> u32 {
                        self.inner.output_spec().len() as u32
                    }

                    #[wasm_bindgen(js_name = "inputNames")]
                    pub fn input_names(&self) -> js_sys::Array {
                        self.inner
                            .input_spec()
                            .iter()
                            .map(|spec| JsValue::from(spec.name.clone()))
                            .collect()
                    }

                    #[wasm_bindgen(js_name = "outputNames")]
                    pub fn output_names(&self) -> js_sys::Array {
                        self.inner
                            .output_spec()
                            .iter()
                            .map(|spec| JsValue::from(spec.name.clone()))
                            .collect()
                    }
                }

                impl Default for #type_name {
                    fn default() -> Self {
                        Self::new()
                    }
                }

                #[wasm_bindgen]
                impl ProcFactory {
                    #[allow(non_snake_case)]
                    #[wasm_bindgen(js_name = #func_name)]
                    pub fn #func_name(&self) -> Proc {
                        Proc { inner: Box::new(<#module::builtins::#proc as Default>::default()) }
                    }
                }
            };
            output.extend(wrapped);
        }

        outputs.push(output);
    }

    outputs
        .into_iter()
        .collect::<proc_macro2::TokenStream>()
        .into()
}
