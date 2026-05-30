extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Expr, Lit, Meta};

#[proc_macro_derive(Iformat)]
pub fn derive_iformat(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse_macro_input!(input as DeriveInput);
    let Data::Enum(ref data_enum) = ast.data else {
        panic!("#[derive(Iformat)] can only be used on enums")
    };
    let name = &ast.ident;
    let re = regex::Regex::new(r"%(?<i>\d)").unwrap();

    let arms = data_enum.variants.iter().map(|variant| {
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

        for attr in &variant.attrs {
            if attr.path().is_ident("doc") {
                if let Meta::NameValue(meta) = &attr.meta {
                    if let Expr::Lit(expr_lit) = &meta.value {
                        if let Lit::Str(s) = &expr_lit.lit {
                            fmt = Some(s.value().trim().to_string());
                        }
                    }
                }
            }
        }

        let fmt = match fmt {
            Some(f) => {
                re.replace_all(&f, "{v$i}").replace(r"\t", "\t")
            },
            None => {
                let args_str = (1..=field_count)
                    .map(|i| format!("{{v{i}}}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} {args_str}", ident.to_string().to_lowercase())
            }
        };

        let fmt_lit = syn::LitStr::new(&fmt, proc_macro::Span::call_site().into());

        if field_count == 0 {
            quote! {
                #name::#ident => f.write_fmt(format_args!(#fmt_lit)),
            }
        } else {
            quote! {
                #name::#ident( #( #vars ),* ) => f.write_fmt(format_args!(#fmt_lit)),
            }
        }
    });

    let generics = ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics ::std::fmt::Display for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #( #arms )*
                }
            }
        }
    }
    .into()
}
