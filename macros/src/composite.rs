use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse::Parse, Expr, ExprClosure, Stmt};

#[derive(Debug)]
pub struct ActionData {
    closure: ExprClosure,
}

impl Parse for ActionData {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let closure: ExprClosure = input.parse()?;

        Ok(Self { closure })
    }
}

impl ActionData {
    pub fn parse_token(&self) -> TokenStream2 {
        if matches!(self.closure.body.as_ref(), &Expr::Async(_)) {
            // Async body
            // no need capture anything
            let body = self.closure.body.as_ref();
            return quote! {
            ::bhv_async::composite::Composite::new_action
                (
                    || Box::pin(#body)
                )
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

        quote! {
            ::bhv_async::composite::Composite::new_action
                (
                    #capture || { #(#mapping_statements)* }
                )
        }
    }
}
