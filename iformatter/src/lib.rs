extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Expr, Lit, Meta};

#[proc_macro_derive(Iformat, attributes(valueType))]
pub fn derive_iformat(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse_macro_input!(input as DeriveInput);
    let Data::Enum(ref data_enum) = ast.data else {
        panic!("#[derive(Iformat)] can only be used on enums")
    };

    let name = &ast.ident;
    let re = regex::Regex::new(r"%(?<i>\d)").unwrap();
    let mut display_arms = vec![];
    let mut dst_arms = vec![];
    let mut src_arms = vec![];
    let mut val_ty = None;
    for attr in ast.attrs.iter() {
        match &attr.meta {
            Meta::List(ml) => {
                if ml.path.is_ident("valueType") {
                    val_ty = Some(ml.tokens.clone());
                    break;
                }
            }
            _ => {}
        }
    }

    for variant in data_enum.variants.iter() {
        let ident = &variant.ident;

        let field_count = match &variant.fields {
            syn::Fields::Unnamed(fields) => fields.unnamed.len(),
            syn::Fields::Unit => 0,
            _ => panic!("only tuple variants supported"),
        };

        let vars: Vec<syn::Ident> = (1..=field_count)
            .map(|i| syn::Ident::new(&format!("v{i}"), proc_macro::Span::call_site().into()))
            .collect();

        let mut fmt = None;
        let mut dst = None;
        let mut src = None;

        for attr in &variant.attrs {
            if attr.path().is_ident("doc") {
                if let Meta::NameValue(meta) = &attr.meta {
                    if let Expr::Lit(expr_lit) = &meta.value {
                        if let Lit::Str(s) = &expr_lit.lit {
                            let s = s.value();
                            let s = s.trim().to_string();
                            let parsed = s.split_once(":");
                            if let Some((cmd, raw)) = parsed {
                                match cmd {
                                    "fmt" => fmt = Some(raw.trim().to_string()),
                                    "dst" => dst = Some(format!("vec![{}]", raw.trim())),
                                    "src" => src = Some(format!("vec![{}]", raw.trim())),
                                    _ => {}
                                }
                            }
                            if s.starts_with("fmt:") {};
                        }
                    }
                }
            }
        }

        println!("{fmt:?}");

        let fmt = match fmt {
            Some(f) => re.replace_all(&f, "{v$i}").replace(r"\t", "\t"),
            None => {
                let args_str = (1..=field_count)
                    .map(|i| format!("{{v{i}}}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} {args_str}", ident.to_string().to_lowercase())
            }
        };

        let dst = dst.unwrap_or("vec![]".into());
        let dst_toks: Expr = syn::parse_str(&re.replace_all(&dst, "{v$i}")).unwrap();

        let src = src.unwrap_or("vec![]".into());
        let src_toks: Expr = syn::parse_str(&re.replace_all(&src, "{v$i}")).unwrap();

        let fmt_lit = syn::LitStr::new(&fmt, proc_macro::Span::call_site().into());

        let display_arm;
        let dst_arm;
        let src_arm;

        if field_count == 0 {
            display_arm = quote! {
                #name::#ident => f.write_fmt(format_args!(#fmt_lit)),
            };

            dst_arm = quote! {
                #name::#ident => #dst_toks,
            };

            src_arm = quote! {
                #name::#ident => #src_toks,
            };
        } else {
            display_arm = quote! {
                #name::#ident( #( #vars ),* ) => f.write_fmt(format_args!(#fmt_lit)),
            };

            dst_arm = quote! {
                #name::#ident( #( #vars ),* ) => #dst_toks,
            };

            src_arm = quote! {
                #name::#ident( #( #vars ),* ) => #src_toks,
            };
        }

        display_arms.push(display_arm);
        dst_arms.push(dst_arm);
        src_arms.push(src_arm);
    }

    let generics = ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let val_ty = val_ty.unwrap();

    quote! {
        impl #impl_generics ::std::fmt::Display for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #( #display_arms )*
                }
            }
        }

        impl #impl_generics #name #ty_generics #where_clause {
            pub fn dsts(&self) -> Vec<&#val_ty> {
                match self {
                    #( #dst_arms )*
                }
            }

            pub fn srcs(&self) -> Vec<&#val_ty> {
                match self {
                    #( #src_arms )*
                }
            }
        }
    }
    .into()
}
