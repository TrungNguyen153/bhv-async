use proc_macro2::{Group, Ident, Punct, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse::Parse, Token};

pub struct GroupBehaviorData {
    actions: Vec<TokenStream2>,
}

impl Parse for GroupBehaviorData {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // eprintln!("{input:#?}");
        let mut actions = vec![];

        loop {
            let Ok(ident) = input.parse::<Ident>() else {
                break;
            };
            if input.parse::<Token![,]>().is_ok() {
                actions.push(quote!(#ident));
                continue;
            }

            // this is not ending by ","
            // maybe this is macro
            // so it will follow syntax
            // Ident Punct Group
            let punct = input.parse::<Punct>()?;
            if punct.as_char() != '!' {
                panic!("Parse error, expect Punct macro => '!'")
            }
            let group = input.parse::<Group>()?;
            // eprintln!("This?? {ident:#?}\n{punct:#?}\n{group:#?}");
            actions.push(quote! {
                #ident #punct #group
            });
            if input.parse::<Token![,]>().is_ok() {
                continue;
            } else {
                break;
            }
        }

        if actions.is_empty() {
            panic!("Childs components empty...")
        }
        // eprintln!("result:\n{actions:#?}");

        Ok(Self { actions })
    }
}

pub enum GroupBehaviorType {
    Sequence,
    PrioritySelector,
}

impl GroupBehaviorData {
    pub fn parse_token(&self, parse_for: GroupBehaviorType) -> TokenStream2 {
        let actions = self.actions.iter().map(|s| {
            quote! {
                #s.into()
            }
        });
        let new_struct_path = match parse_for {
            GroupBehaviorType::Sequence => quote! {::bhv_async::common_behaviors::Sequence::new},
            GroupBehaviorType::PrioritySelector => {
                quote! {::bhv_async::common_behaviors::PrioritySelector::new}
            }
        };
        quote! {
        #new_struct_path
        (
            [#(#actions,)*]
        )
        }
    }
}
