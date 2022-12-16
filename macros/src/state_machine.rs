use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    Error, Expr, ExprCall, Ident, ItemMod, Token,
};

#[derive(Debug, Default)]
struct Arguments {
    initialize: Option<ExprCall>,
}

impl Parse for Arguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut result = Self::default();
        if !input.is_empty() {
            let arguments: Punctuated<Argument, Token![,]> =
                input.parse_terminated(Argument::parse)?;
            for argument in arguments {
                match argument.key.to_string().as_str() {
                    "initialize" => {
                        if result.initialize.is_some() {
                            return Err(Error::new(argument.key.span(), "duplicate argument"));
                        }
                        result.initialize = match argument.value {
                            Expr::Call(expr) => Some(expr),
                            value => {
                                return Err(Error::new(value.span(), "expected call expression"))
                            }
                        }
                    }
                    _ => return Err(Error::new(argument.key.span(), "unrecognized option")),
                }
            }
        }
        Ok(result)
    }
}

struct Argument {
    key: Ident,
    value: Expr,
}

impl Parse for Argument {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key = input.parse()?;
        input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(Self { key, value })
    }
}

pub fn make_state_machine(args: TokenStream, item: TokenStream) -> TokenStream {
    let arguments = parse_macro_input!(args as Arguments);
    let body = parse_macro_input!(item as ItemMod);

    println!("{:#?}", arguments);

    quote!(#body).into()
}

// TODO: switch to function-style macro to keep compatible syntax.
