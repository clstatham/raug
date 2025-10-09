use proc_macro::TokenStream;
use processor_attribute::processor_attribute;
use quote::quote;
use syn::punctuated::Punctuated;

mod processor_attribute;
mod wrap_wasm;

#[proc_macro_attribute]
pub fn processor(attr: TokenStream, item: TokenStream) -> TokenStream {
    processor_attribute(attr, item)
}

#[proc_macro]
pub fn wrap_wasm(input: TokenStream) -> TokenStream {
    wrap_wasm::wrap_wasm(input)
}

/// Returns the MIDI note constant for the given note name and octave.
///
/// # Examples
///
/// ```
/// let note = raug_macros::note!["C4"];
/// assert_eq!(note, 60);
///
/// let note = raug_macros::note!["C#4"];
/// assert_eq!(note, 61);
///
/// let note = raug_macros::note!["Bb3"];
/// assert_eq!(note, 58);
/// ```
#[proc_macro]
pub fn note(input: TokenStream) -> TokenStream {
    let input: syn::LitStr = syn::parse(input).unwrap();
    let input = input.value();

    let note = parse_note(&input);

    let output = quote! {
        #note
    };

    output.into()
}

/// Returns an array of MIDI notes for the given note names.
/// The note names should be separated by whitespace.
///
/// # Examples
///
/// ```
/// let notes = raug_macros::note_array!["C4 Db4 E4"];
/// assert_eq!(notes, [60, 61, 64]);
/// ```
#[proc_macro]
pub fn note_array(input: TokenStream) -> TokenStream {
    let input: syn::LitStr = syn::parse(input).unwrap();
    let input = input.value();

    let notes = input.split_whitespace().map(parse_note);

    let output = quote! {
        [#(#notes),*]
    };

    output.into()
}

fn parse_note(input: &str) -> u8 {
    let input = input.trim();
    let input = input.to_uppercase();

    let mut chars = input.chars();

    let mut note: i8 = match chars.next().expect("Invalid note: empty input") {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => panic!("Invalid note: invalid note name"),
    };

    let mut octave: i8 = 0;
    let mut stop = false;
    while !stop {
        let next = chars.next();
        match next {
            Some('#') => note += 1, // keep going
            Some('B') => note -= 1, // keep going
            Some('0'..='9') => {
                let num = next.unwrap().to_digit(10).unwrap() as i8;
                octave = num + 1;
                stop = true;
            }
            Some('-') => {
                let num = chars.next().unwrap().to_digit(10).unwrap() as i8;
                octave = -num + 1;
                stop = true;
            }
            None => {
                stop = true;
            }
            _ => {
                panic!("Invalid note: unexpected character");
            }
        }
    }

    let octave = (octave)
        .checked_mul(12)
        .expect("Invalid note: octave out of range");
    let note = note
        .checked_add(octave)
        .expect("Invalid note: note out of range");

    note as u8
}

struct IterProcIoAs {
    inputs: syn::Ident,
    input_types: Punctuated<syn::Type, syn::Token![,]>,
    outputs: syn::Ident,
    output_types: Punctuated<syn::Type, syn::Token![,]>,
}

impl syn::parse::Parse for IterProcIoAs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let inputs = input.parse()?;
        input.parse::<syn::Token![as]>()?;
        let input_types;
        syn::bracketed!(input_types in input);
        let input_types = input_types.parse_terminated(syn::Type::parse, syn::Token![,])?;
        input.parse::<syn::Token![,]>()?;
        let outputs = input.parse()?;
        input.parse::<syn::Token![as]>()?;
        let output_types;
        syn::bracketed!(output_types in input);
        let output_types = output_types.parse_terminated(syn::Type::parse, syn::Token![,])?;
        Ok(Self {
            inputs,
            outputs,
            input_types,
            output_types,
        })
    }
}

#[proc_macro]
pub fn iter_proc_io_as(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as IterProcIoAs);

    let inputs = input.inputs;
    let outputs = input.outputs;

    let input_count = input.input_types.len();
    let output_count = input.output_types.len();

    let mut input_idents = vec![];
    for i in 0..input_count {
        let ident = syn::Ident::new(&format!("in{}", i), proc_macro2::Span::call_site());
        input_idents.push(ident);
    }

    let mut output_idents = vec![];
    for i in 0..output_count {
        let ident = syn::Ident::new(&format!("out{}", i), proc_macro2::Span::call_site());
        output_idents.push(ident);
    }

    let start = quote! {
        let raug::processor::io::ProcessorInputs {
            input_specs,
            inputs,
            env,
            ..
        } = #inputs;

        let [#(#input_idents),*] = inputs else {
            panic!("Expected {} inputs, got {}", #input_count, inputs.len());
        };

        let raug::processor::io::ProcessorOutputs {
            output_spec,
            outputs,
            mode,
            ..
        } = #outputs;

        let [#(#output_idents),*] = outputs else {
            panic!("Expected {} outputs, got {}", #output_count, outputs.len());
        };
    };

    let mut chunks = vec![];

    for (i, (input_ident, input_typ)) in input_idents
        .iter()
        .zip(input.input_types.iter())
        .enumerate()
    {
        if let syn::Type::Path(path) = input_typ {
            if path.path.get_ident().unwrap() == "Any" {
                let chunk = quote! {
                    raug::processor::io::ProcessorInputs::new(
                        std::slice::from_ref(&input_specs[#i]),
                        std::slice::from_ref(#input_ident),
                        env,
                    ).iter_input(0)
                };
                chunks.push(chunk);
                continue;
            }
        }
        let chunk = quote! {
            raug::processor::io::ProcessorInputs::new(
                std::slice::from_ref(&input_specs[#i]),
                std::slice::from_ref(#input_ident),
                env,
            ).iter_input_as::<#input_typ>(0)?
        };
        chunks.push(chunk);
    }

    for (i, (output_ident, output_typ)) in output_idents
        .iter()
        .zip(input.output_types.iter())
        .enumerate()
    {
        if let syn::Type::Path(path) = output_typ {
            if path.path.get_ident().unwrap() == "Any" {
                let chunk = quote! {
                    raug::processor::io::ProcessorOutputs::new(
                        std::slice::from_ref(&output_spec[#i]),
                        std::slice::from_mut(#output_ident),
                        mode,
                    ).iter_output_mut(0)
                };
                chunks.push(chunk);
                continue;
            }
        }
        let chunk = quote! {
            raug::processor::io::ProcessorOutputs::new(
                std::slice::from_ref(&output_spec[#i]),
                std::slice::from_mut(#output_ident),
                mode,
            ).iter_output_mut_as::<#output_typ>(0)?
        };
        chunks.push(chunk);
    }

    let output = quote! {{
        #start

        raug::__itertools::izip!(#(#chunks),*)
    }};

    output.into()
}
