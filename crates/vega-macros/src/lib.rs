use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{
    parse_macro_input, Expr, ExprArray, ExprLit, ExprPath, Ident, ItemFn, Lit, LitStr, Meta, Token,
};

struct PageArgs {
    mode: LitStr,
    middleware: Vec<String>,
    revalidate: Option<u64>,
}

impl Default for PageArgs {
    fn default() -> Self {
        Self {
            mode: LitStr::new("ssr", proc_macro2::Span::call_site()),
            middleware: Vec::new(),
            revalidate: None,
        }
    }
}

impl Parse for PageArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut args = PageArgs::default();
        let metas = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

        for meta in metas {
            let Meta::NameValue(name_value) = meta else {
                return Err(syn::Error::new_spanned(meta, "unsupported page argument"));
            };

            if name_value.path.is_ident("mode") {
                let Expr::Lit(ExprLit {
                    lit: Lit::Str(mode),
                    ..
                }) = name_value.value
                else {
                    return Err(syn::Error::new_spanned(name_value, "mode must be a string"));
                };
                args.mode = mode;
            } else if name_value.path.is_ident("revalidate") {
                let Expr::Lit(ExprLit {
                    lit: Lit::Int(value),
                    ..
                }) = name_value.value
                else {
                    return Err(syn::Error::new_spanned(
                        name_value,
                        "revalidate must be an integer",
                    ));
                };
                args.revalidate = Some(value.base10_parse::<u64>()?);
            } else if name_value.path.is_ident("middleware") {
                let Expr::Array(ExprArray { elems, .. }) = name_value.value else {
                    return Err(syn::Error::new_spanned(
                        name_value,
                        "middleware must be an array",
                    ));
                };

                let mut middleware = Vec::new();
                for elem in elems {
                    let Expr::Path(ExprPath { path, .. }) = elem else {
                        return Err(syn::Error::new_spanned(
                            elem,
                            "middleware item must be a path",
                        ));
                    };
                    middleware.push(quote!(#path).to_string().replace(' ', ""));
                }
                args.middleware = middleware;
            } else {
                return Err(syn::Error::new_spanned(name_value, "unknown page argument"));
            }
        }

        Ok(args)
    }
}

#[proc_macro_attribute]
pub fn page(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as PageArgs);
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = input_fn.sig.ident.clone();

    let mode = args.mode.value();
    let mode_variant = match mode.as_str() {
        "ssr" => quote!(vega::core::RenderMode::Ssr),
        "ssg" => quote!(vega::core::RenderMode::Ssg),
        "csr" => quote!(vega::core::RenderMode::Csr),
        "isr" => quote!(vega::core::RenderMode::Isr),
        _ => {
            return syn::Error::new_spanned(args.mode, "mode must be one of: ssr, ssg, csr, isr")
                .to_compile_error()
                .into();
        }
    };

    let middleware = args
        .middleware
        .iter()
        .map(|m| LitStr::new(m, proc_macro2::Span::call_site()))
        .collect::<Vec<_>>();

    let revalidate_expr = if let Some(revalidate) = args.revalidate {
        quote!(Some(#revalidate))
    } else {
        quote!(None)
    };

    let expanded = quote! {
        #input_fn

        #[doc(hidden)]
        pub const __VEGA_PAGE_META: vega::core::PageMeta = vega::core::PageMeta {
            mode: #mode_variant,
            revalidate: #revalidate_expr,
            middleware: &[#(#middleware),*],
            component_name: stringify!(#fn_name),
            file: file!(),
        };
    };

    expanded.into()
}

#[proc_macro_attribute]
pub fn layout(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    if input_fn.sig.inputs.len() != 1 {
        return syn::Error::new_spanned(
            &input_fn.sig,
            "#[vega::layout] expects exactly one argument: children",
        )
        .to_compile_error()
        .into();
    }

    let fn_name = input_fn.sig.ident.clone();
    let expanded = quote! {
        #input_fn

        #[doc(hidden)]
        pub const __VEGA_LAYOUT_COMPONENT: &str = stringify!(#fn_name);
    };
    expanded.into()
}

#[proc_macro_attribute]
pub fn server_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let cache_seconds = parse_cache_seconds(attr);
    let cache_expr = match cache_seconds {
        Ok(Some(seconds)) => quote!(Some(#seconds)),
        Ok(None) => quote!(None),
        Err(err) => return err.to_compile_error().into(),
    };

    let fn_name = input_fn.sig.ident.clone();
    let cache_const = format_ident!(
        "__VEGA_SERVER_FN_CACHE_{}",
        fn_name.to_string().to_ascii_uppercase()
    );

    let expanded = quote! {
        #input_fn

        #[doc(hidden)]
        pub const #cache_const: Option<u64> = #cache_expr;
    };
    expanded.into()
}

#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    api_macro(attr, item, "Get")
}

#[proc_macro_attribute]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    api_macro(attr, item, "Post")
}

#[proc_macro_attribute]
pub fn put(attr: TokenStream, item: TokenStream) -> TokenStream {
    api_macro(attr, item, "Put")
}

#[proc_macro_attribute]
pub fn patch(attr: TokenStream, item: TokenStream) -> TokenStream {
    api_macro(attr, item, "Patch")
}

#[proc_macro_attribute]
pub fn delete(attr: TokenStream, item: TokenStream) -> TokenStream {
    api_macro(attr, item, "Delete")
}

fn api_macro(attr: TokenStream, item: TokenStream, method_variant: &str) -> TokenStream {
    let middleware = match parse_middleware_only(attr) {
        Ok(values) => values,
        Err(err) => return err.to_compile_error().into(),
    };

    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = input_fn.sig.ident.clone();
    let meta_name = format_ident!(
        "__VEGA_API_META_{}",
        fn_name.to_string().to_ascii_uppercase()
    );
    let middleware_lits = middleware
        .iter()
        .map(|m| LitStr::new(m, proc_macro2::Span::call_site()))
        .collect::<Vec<_>>();

    let method_ident = Ident::new(method_variant, proc_macro2::Span::call_site());
    let expanded = quote! {
        #input_fn

        #[doc(hidden)]
        pub const #meta_name: vega::core::ApiMeta = vega::core::ApiMeta {
            method: vega::core::HttpMethod::#method_ident,
            middleware: &[#(#middleware_lits),*],
            fn_name: stringify!(#fn_name),
            file: file!(),
        };
    };

    expanded.into()
}

fn parse_cache_seconds(attr: TokenStream) -> syn::Result<Option<u64>> {
    if attr.is_empty() {
        return Ok(None);
    }

    let metas = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(attr.into())?;
    for meta in metas {
        let Meta::NameValue(value) = meta else {
            return Err(syn::Error::new_spanned(meta, "invalid server_fn argument"));
        };

        if value.path.is_ident("cache") {
            let Expr::Lit(ExprLit {
                lit: Lit::Int(seconds),
                ..
            }) = value.value
            else {
                return Err(syn::Error::new_spanned(value, "cache must be integer"));
            };
            return Ok(Some(seconds.base10_parse::<u64>()?));
        }
    }

    Ok(None)
}

fn parse_middleware_only(attr: TokenStream) -> syn::Result<Vec<String>> {
    if attr.is_empty() {
        return Ok(Vec::new());
    }

    let metas = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(attr.into())?;
    if let Some(meta) = metas.into_iter().next() {
        let Meta::NameValue(value) = meta else {
            return Err(syn::Error::new_spanned(meta, "invalid API macro argument"));
        };
        if !value.path.is_ident("middleware") {
            return Err(syn::Error::new_spanned(
                value.path,
                "only middleware is supported",
            ));
        }

        let Expr::Array(ExprArray { elems, .. }) = value.value else {
            return Err(syn::Error::new_spanned(
                value,
                "middleware must be an array",
            ));
        };

        let mut middleware = Vec::new();
        for elem in elems {
            let Expr::Path(ExprPath { path, .. }) = elem else {
                return Err(syn::Error::new_spanned(
                    elem,
                    "middleware item must be a path",
                ));
            };
            middleware.push(quote!(#path).to_string().replace(' ', ""));
        }

        return Ok(middleware);
    }

    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_page_args_defaults() {
        let parsed = syn::parse_str::<PageArgs>("").expect("parse");
        assert_eq!(parsed.mode.value(), "ssr");
        assert!(parsed.middleware.is_empty());
        assert_eq!(parsed.revalidate, None);
    }

    #[test]
    fn parse_page_args_full() {
        let parsed = syn::parse_str::<PageArgs>(
            "mode = \"ssg\", middleware = [auth::require_auth, auth::admin], revalidate = 60",
        )
        .expect("parse");

        assert_eq!(parsed.mode.value(), "ssg");
        assert_eq!(parsed.revalidate, Some(60));
        assert_eq!(parsed.middleware.len(), 2);
    }
}
