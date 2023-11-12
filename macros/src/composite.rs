use proc_macro2::{Literal, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse::Parse, Expr, ExprClosure, Stmt, Token};

#[derive(Debug)]
pub struct ActionData {
    action_name: Option<Literal>,
    closure: ExprClosure,
}

impl Parse for ActionData {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let action_name = input.parse::<Literal>().ok();
        if action_name.is_some() {
            input.parse::<Token![,]>()?;
        }
        let closure: ExprClosure = input.parse()?;

        Ok(Self {
            closure,
            action_name,
        })
    }
}

impl ActionData {
    pub fn parse_token(&self) -> TokenStream2 {
        if matches!(self.closure.body.as_ref(), &Expr::Async(_)) {
            // Async body
            // no need capture anything
            let body = self.closure.body.as_ref();
            if let Some(action_name) = &self.action_name {
                return quote! {
                ::bhv_async::composite::Composite::new
                    (
                        #action_name, || Box::pin(#body)
                    )
                };
            } else {
                return quote! {
                ::bhv_async::composite::Composite::new_action
                    (
                        || Box::pin(#body)
                    )
                };
            };
        }

        // more thing than async
        let capture = if self.closure.capture.is_some() {
            quote!(move)
        } else {
            quote!()
        };

        let Expr::Block(expr_block) = self.closure.body.as_ref() else {
            panic!("Expect body Block")
        };
        let stmts = &*expr_block.block.stmts;
        let mapping_statements = stmts.iter().map(|stmt| match stmt {
            Stmt::Expr(Expr::Async(expr_async), _) => {
                quote! {
                    Box::pin(#expr_async)
                }
            }
            Stmt::Expr(Expr::Return(expr_return), _) => {
                // with_return
                let inner_return = expr_return
                    .expr
                    .as_ref()
                    .expect("Should be Some Expr::Async")
                    .as_ref();
                quote! {
                    return Box::pin(#inner_return);
                }
            }
            _ => {
                quote! {
                    #stmt
                }
            }
        });

        if let Some(action_name) = &self.action_name {
            quote! {
                ::bhv_async::composite::Composite::new
                    (
                        #action_name, #capture || { #(#mapping_statements)* }
                    )
            }
        } else {
            quote! {
                ::bhv_async::composite::Composite::new_action
                    (
                        #capture || { #(#mapping_statements)* }
                    )
            }
        }
    }
}
