#![feature(proc_macro_expand)]
#![allow(non_snake_case)]
mod composite;
mod decorator;
mod groups;

use groups::*;
use proc_macro::TokenStream;
use syn::parse_macro_input;

use composite::ActionData;

use self::decorator::DecoratorData;

/// Example
/// let action = Action! {
///     || async {
///         sleep(Duration::from_secs(1)).await;
///         RunStatus::Success
///     }
/// };

/// let action_with_capture = Action! {
///     move || {
///         let action_move = action.clone();
///         return async move {
///             (action_move.task_production)().await;
///             RunStatus::Success
///         }
///     }
/// };
#[proc_macro]
pub fn Action(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ActionData);
    input.parse_token().into()
}

/// let _action_with_capture = Action! {
///     move || {
///         let action_move = action.clone();
///         return async move {
///             (action_move.task_production)().await;
///             RunStatus::Success
///         }
///     }
/// };
/// let seq = Sequence! {
///     _action_with_capture,
///     Action! {
///         || async {
///             sleep(Duration::from_secs(1)).await;
///             RunStatus::Success
///         }
///     },
///     Action! {
///         || async {
///             sleep(Duration::from_secs(1)).await;
///             RunStatus::Success
///         }
///     },
///     Sequence! {
///             Action! {
///                 || async {
///                     sleep(Duration::from_secs(1)).await;
///                     RunStatus::Failure
///                 }
///             },
///             Action! {
///                 || async {
///                     sleep(Duration::from_secs(1)).await;
///                     RunStatus::Success
///                 }
///             },
///         }
/// };
#[proc_macro]
pub fn Sequence(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GroupBehaviorData);
    input.parse_token(GroupBehaviorType::Sequence).into()
}

#[proc_macro]
pub fn PrioritySelector(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GroupBehaviorData);
    input
        .parse_token(GroupBehaviorType::PrioritySelector)
        .into()
}

#[proc_macro]
pub fn Decorator(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DecoratorData);
    input
        .parse_token(decorator::DecoratorType::Decorator)
        .into()
}

#[proc_macro]
pub fn DecoratorContinue(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DecoratorData);
    input
        .parse_token(decorator::DecoratorType::DecoratorContinue)
        .into()
}
