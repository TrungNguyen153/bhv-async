use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse::Parse, ExprClosure, Token};

pub struct DecoratorData {
    condition: ExprClosure,
    task_creation: Option<ExprClosure>,
    or_another_composite: Option<Ident>,
}

impl Parse for DecoratorData {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let condition = input.parse::<ExprClosure>()?;
        // eprintln!("can work ?? {condition:#?}");
        input.parse::<Token![,]>()?;

        if let Ok(task_creation) = input.parse::<ExprClosure>() {
            return Ok(Self {
                condition,
                task_creation: Some(task_creation),
                or_another_composite: None,
            });
        }

        if let Ok(or_another_composite) = input.parse::<Ident>() {
            return Ok(Self {
                condition,
                task_creation: None,
                or_another_composite: Some(or_another_composite),
            });
        }
        panic!("Decorator have no TaskCreation or another Composite")
    }
}

pub enum DecoratorType {
    Decorator,
    DecoratorContinue,
}

impl DecoratorData {
    pub fn parse_token(&self, parse_for: DecoratorType) -> TokenStream {
        let new_struct_path = match parse_for {
            DecoratorType::Decorator => quote!(::bhv_async::common_behaviors::Decorator::new),
            DecoratorType::DecoratorContinue => {
                quote!(::bhv_async::common_behaviors::DecoratorContinue::new)
            }
        };
        let condition = &self.condition;
        if let Some(task_creation) = self.task_creation.as_ref().map(|i| quote!(#i)) {
            quote! {
                #new_struct_path
                    (
                        #condition, Action!(#task_creation)
                    )
            }
        } else {
            let another_composite = self.or_another_composite.as_ref().unwrap();
            quote! {
                #new_struct_path
                    (
                        #condition, #another_composite
                    )
            }
        }
    }
}
