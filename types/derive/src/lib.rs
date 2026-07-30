use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, GenericArgument, PathArguments, Type,
};

/// Generates a `{StructName}Patch` struct from the annotated struct.
///
/// Every `Option<T>` field becomes `Option<Option<T>>` with serde
/// `double_option` handling. Non-Option fields become `Option<T>`.
///
/// Serialization semantics per field:
/// - `None`           → field omitted (via `skip_serializing_if`)
/// - `Some(None)`     → `"field": null`
/// - `Some(Some(v))`  → `"field": <value>`
///
/// # Attributes
///
/// - `#[patch_name(CustomName)]` — override struct name (default: `{Struct}Patch`)
/// - `#[patch_derives(Clone, Debug, ...)]` — override derives
///   (default: `Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq`)
/// - `#[patch_double_option_mod(path)]` — module path for the double_option
///   serde helper (default: `double_option`)
///
/// # Usage
///
/// ```rust
/// #[derive(GeneratePatch)]
/// pub struct Details {
///     priority: Option<Priority>,
///     context: Option<String>,
/// }
/// ```
///
/// Generates:
///
/// ```rust
/// #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
/// pub struct DetailsPatch {
///     #[serde(default, with = "double_option", skip_serializing_if = "Option::is_none")]
///     pub priority: Option<Option<Priority>>,
///     #[serde(default, with = "double_option", skip_serializing_if = "Option::is_none")]
///     pub context: Option<Option<String>>,
/// }
/// ```
#[proc_macro_derive(
    GeneratePatch,
    attributes(
        patch_name,
        patch_derives,
        patch_double_option_mod
    )
)]
pub fn generate_patch(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let source_name = &input.ident;
    let vis = &input.vis;

    let patch_name = input
        .attrs
        .iter()
        .find(|a| a.path().is_ident("patch_name"))
        .map(|a| {
            a.parse_args::<syn::Ident>()
                .expect("patch_name must be an identifier")
        })
        .unwrap_or_else(|| format_ident!("{}Patch", source_name));

    let double_option_mod: syn::Path = input
        .attrs
        .iter()
        .find(|a| a.path().is_ident("patch_double_option_mod"))
        .map(|a| {
            a.parse_args::<syn::Path>()
                .expect("patch_double_option_mod must be a path")
        })
        .unwrap_or_else(|| syn::parse_str("double_option").unwrap());

    let double_option_str = quote!(#double_option_mod).to_string();

    let extra_derives: Option<syn::punctuated::Punctuated<syn::Path, syn::Token![,]>> = input
        .attrs
        .iter()
        .find(|a| a.path().is_ident("patch_derives"))
        .map(|a| {
            a.parse_args_with(syn::punctuated::Punctuated::parse_terminated)
                .expect("patch_derives must be a comma-separated list of paths")
        });

    let derives = if let Some(extra) = extra_derives {
        quote! { #[derive(#extra)] }
    } else {
        quote! { #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)] }
    };

    let fields = extract_named_fields(&input.data);

    let patch_fields = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;

        let (patch_ty, needs_double_option) = if let Some(inner) = extract_option_inner(ty) {
            (quote! { Option<Option<#inner>> }, true)
        } else {
            (quote! { Option<#ty> }, false)
        };

        let serde_attr = if needs_double_option {
            quote! { #[serde(default, with = #double_option_str, skip_serializing_if = "Option::is_none")] }
        } else {
            quote! { #[serde(default, skip_serializing_if = "Option::is_none")] }
        };

        // `serde(with = "double_option")` points schemars at a serde helper
        // *module*, not a type — schemars_derive tries to resolve it as a
        // type for JSON-Schema generation and fails. Point it at the real
        // `Option<Option<T>>` shape explicitly instead. Only relevant when
        // `schemars::JsonSchema` is actually derived on the patch struct
        // (gated behind the `mcp` feature in `kid-types`).
        let schemars_attr = if needs_double_option {
            let with_str = quote!(#patch_ty).to_string();
            quote! { #[cfg_attr(feature = "mcp", schemars(with = #with_str))] }
        } else {
            quote! {}
        };

        quote! {
            #serde_attr
            #schemars_attr
            #vis #name: #patch_ty
        }
    });

    let expanded = quote! {
        #derives
        #vis struct #patch_name {
            #(#patch_fields,)*
        }
    };

    expanded.into()
}

/// Derives `apply_patch(&mut self, patch: PatchStruct)` on the annotated struct.
///
/// # Usage
///
/// ```rust
/// #[derive(Patchable)]
/// #[patch_type(DetailsPatch)]
/// pub struct Details {
///     priority: Option<Priority>,
///     due_date: Option<Date>,
/// }
/// ```
///
/// - `None`           → field unchanged
/// - `Some(None)`     → field nulled
/// - `Some(Some(v))`  → field set to `Some(v)`
#[proc_macro_derive(Patchable, attributes(patch_type))]
pub fn derive_patchable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let target_name = &input.ident;

    let patch_ident: syn::Ident = input
        .attrs
        .iter()
        .find(|a| a.path().is_ident("patch_type"))
        .expect("#[derive(Patchable)] requires a #[patch_type(PatchStruct)] attribute")
        .parse_args()
        .expect("#[patch_type(...)] must contain a single type identifier");

    let fields = extract_named_fields(&input.data);

    let assignments = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        quote! {
            if let Some(v) = patch.#name {
                self.#name = v;
            }
        }
    });

    let expanded = quote! {
        impl #target_name {
            pub fn apply_patch(&mut self, patch: #patch_ident) {
                #(#assignments)*
            }
        }
    };

    expanded.into()
}

fn extract_named_fields(
    data: &Data,
) -> &syn::punctuated::Punctuated<syn::Field, syn::Token![,]> {
    match data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => panic!("only structs with named fields are supported"),
        },
        _ => panic!("only structs are supported"),
    }
}

fn extract_option_inner(ty: &Type) -> Option<&GenericArgument> {
    if let Type::Path(tp) = ty {
        let seg = tp.path.segments.last()?;
        if seg.ident != "Option" {
            return None;
        }
        if let PathArguments::AngleBracketed(args) = &seg.arguments {
            return args.args.first();
        }
    }
    None
}
