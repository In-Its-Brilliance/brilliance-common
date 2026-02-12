use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, ReturnType};

#[proc_macro_attribute]
pub fn event_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_block = &input_fn.block;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;
    let mut inputs_iter = fn_inputs.iter();
    let (event_pat, event_type) = match inputs_iter.next() {
        Some(FnArg::Typed(pat_type)) => (&pat_type.pat, &pat_type.ty),
        _ => {
            return syn::Error::new_spanned(&input_fn.sig, "event_handler requires at least one typed argument")
                .to_compile_error()
                .into();
        }
    };
    let extra_args: Vec<_> = inputs_iter
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                Some((&pat_type.pat, &pat_type.ty))
            } else {
                None
            }
        })
        .collect();
    let extra_lets: Vec<_> = extra_args
        .iter()
        .map(|(pat, ty)| {
            quote! { let #pat: #ty = Default::default(); }
        })
        .collect();
    let extra_pats: Vec<_> = extra_args.iter().map(|(pat, _)| pat).collect();
    let extra_params: Vec<_> = extra_args.iter().map(|(pat, ty)| quote! { #pat: #ty }).collect();
    let has_return = !matches!(fn_output, ReturnType::Default);
    let inner_call = if extra_pats.is_empty() {
        quote! { __inner_fn(#event_pat) }
    } else {
        quote! { __inner_fn(#event_pat, #(#extra_pats),*) }
    };
    let output_handling = if has_return {
        quote! {
            match #inner_call {
                Ok(val) => {
                    let __output_str = common::serde_json::to_string(&val)
                        .expect("Failed to serialize return value");
                    common::extism_pdk::output(&__output_str)
                        .expect("Failed to write output");
                }
                Err(e) => {
                    common::extism_pdk::log!(
                        common::extism_pdk::LogLevel::Error,
                        "Event error: {:?}",
                        e
                    );
                    return 1;
                }
            }
        }
    } else {
        quote! {
            #inner_call;
        }
    };
    let expanded = quote! {
        #[no_mangle]
        #fn_vis extern "C" fn #fn_name() -> i32 {
            fn __inner_fn(#event_pat: #event_type, #(#extra_params),*) #fn_output #fn_block
            let __input: String = match common::extism_pdk::input() {
                Ok(s) => s,
                Err(e) => {
                    common::extism_pdk::log!(common::extism_pdk::LogLevel::Error, "Input error: {:?}", e);
                    return 1;
                }
            };
            let #event_pat: #event_type = match common::serde_json::from_str(&__input) {
                Ok(e) => e,
                Err(e) => {
                    common::extism_pdk::log!(common::extism_pdk::LogLevel::Error, "Deserialize error: {:?}", e);
                    return 2;
                }
            };
            #(#extra_lets)*
            #output_handling
            0
        }
    };
    TokenStream::from(expanded)
}
