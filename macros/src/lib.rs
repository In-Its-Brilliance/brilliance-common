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

    let event_type = match fn_inputs.first() {
        Some(FnArg::Typed(pat_type)) => &pat_type.ty,
        _ => {
            return syn::Error::new_spanned(
                &input_fn.sig,
                "event_handler requires exactly one typed argument",
            )
            .to_compile_error()
            .into();
        }
    };

    let has_return = !matches!(fn_output, ReturnType::Default);

    let output_handling = if has_return {
        quote! {
            let __result = __inner_fn(__event);
            let __output_str = common::serde_json::to_string(&__result)
                .expect("Failed to serialize return value");
            common::extism_pdk::output(&__output_str)
                .expect("Failed to write output");
        }
    } else {
        quote! {
            __inner_fn(__event);
        }
    };

    let expanded = quote! {
        #[no_mangle]
        #fn_vis extern "C" fn #fn_name() -> i32 {
            fn __inner_fn(__event: #event_type) #fn_output #fn_block

            let __input: String = match common::extism_pdk::input() {
                Ok(s) => s,
                Err(e) => {
                    common::extism_pdk::log!(common::extism_pdk::LogLevel::Error, "Input error: {:?}", e);
                    return 1;
                }
            };

            let __event: #event_type = match common::serde_json::from_str(&__input) {
                Ok(e) => e,
                Err(e) => {
                    common::extism_pdk::log!(common::extism_pdk::LogLevel::Error, "Deserialize error: {:?}", e);
                    return 2;
                }
            };

            #output_handling
            0
        }
    };

    TokenStream::from(expanded)
}
