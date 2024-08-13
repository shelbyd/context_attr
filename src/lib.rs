use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Expr, ItemFn};

#[cfg(feature = "eyre")]
mod eyre;

#[cfg(feature = "eyre")]
#[proc_macro_attribute]
pub fn eyre(attr: TokenStream, item: TokenStream) -> TokenStream {
    use syn::{punctuated::Punctuated, token::Comma, FnArg};

    let input = parse_macro_input!(item as ItemFn);
    let create_message = parse_macro_input!(attr as Expr);

    let sig = &input.sig;
    let args = input
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Receiver(r) => {
                let self_ = &r.self_token;
                quote! { #self_ }
            }
            FnArg::Typed(typed) => {
                let pat = &typed.pat;
                quote! { #pat }
            }
        })
        .collect::<Punctuated<_, Comma>>();

    let do_create = match &create_message {
        Expr::Closure(c) => {
            let args = args
                .iter()
                .map(|a| quote! { &#a })
                .collect::<Punctuated<_, Comma>>();
            quote! { (#c)(#args) }
        }
        e => e.to_token_stream(),
    };

    let block = &input.block;

    // Create the output tokens
    let expanded = quote! {
        #sig {
            let message = #do_create;

            let mut inner = move || #block;

            let result = inner();
            eyre::WrapErr::<_, _>::context(result, message)
        }
    };

    // Convert the quote tokens back into a token stream
    TokenStream::from(expanded)
}
