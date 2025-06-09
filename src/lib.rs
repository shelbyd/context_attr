//! Add a context to the result returned from the annotated function, regardless of return path.
//!
//! ```
//! #[context_attr::anyhow("Doing thing")]
//! fn do_thing() -> anyhow::Result<()> {
//!   if true {
//!     anyhow::bail!("Can't do thing");
//!   }
//!   anyhow::bail!("Another error path");
//! }
//! ```
//!
//! This is most useful when there are multiple error points in a function, and annotating the context of the function at each point adds too much noise.
//!
//! ## Format
//!
//! You can call [std::format] to include function arguments in the message.
//!
//! ```
//! #[context_attr::anyhow(format!("The number: {x}"))]
//! fn add_one(x: u32) -> anyhow::Result<()> { todo!() }
//! ```
//!
//! **Note**: that the format string is eagerly created at the beginning of the function. There currently isn't a way around this, as arguments to the function may be moved.
//!
//! ## Works with
//!
//! The annotation works with:
//!
//! - Async functions
//! - Functions in impl blocks

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Expr, ItemFn};

/// Add the provided context to an anyhow::Result.
#[cfg(feature = "anyhow")]
#[proc_macro_attribute]
pub fn anyhow(attr: TokenStream, item: TokenStream) -> TokenStream {
    wrap_with(attr, item, quote! { anyhow })
}

#[allow(unused)]
fn wrap_with(
    attr: TokenStream,
    item: TokenStream,
    crate_: proc_macro2::TokenStream,
) -> TokenStream {
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
    let vis = &input.vis;

    let inner = match &input.sig.asyncness {
        Some(_) => quote! { async #block.await },
        None => quote! { (|| #block)() },
    };

    // Create the output tokens
    let expanded = quote! {
        #vis #sig {
            let message = #do_create;

            let inner: #crate_::Result<_> = #inner;
            match inner {
                Ok(t) => Ok(t),
                Err(e) => {
                    #crate_::Context::<_, _>::context(Err(e), message)
                }
            }
        }
    };

    // Convert the quote tokens back into a token stream
    TokenStream::from(expanded)
}

#[cfg(feature = "eyre")]
#[proc_macro_attribute]
pub fn eyre(attr: TokenStream, item: TokenStream) -> TokenStream {
    wrap_with(attr, item, quote! { eyre })
}
