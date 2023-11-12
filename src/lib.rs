#[macro_use]
pub mod composite;
pub mod common_behaviors;
#[cfg(feature = "macros")]
pub mod macros {
    pub use bhv_async_macros::*;
}
/// Re-export all type in bhv-async
pub mod prelude {
    pub use crate::common_behaviors::*;
    pub use crate::composite::*;
    pub use crate::RunStatus;

    #[cfg(feature = "macros")]
    pub use crate::macros::*;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    Success,
    #[default]
    Failure,
}

#[cfg(test)]
mod tests {
    pub use crate::prelude::*;

    #[tokio::test]
    pub async fn run_tree() {
        let seq = Sequence::new([
            Composite::new_action(|| Box::pin(async { RunStatus::Success })),
            DecoratorContinue::new(
                || true,
                Composite::new_action(|| Box::pin(async { RunStatus::Success })),
            )
            .into(),
            Sequence::new([
                Composite::new_action(|| Box::pin(async { RunStatus::Success })),
                DecoratorContinue::new(
                    || true,
                    Composite::new_action(|| Box::pin(async { RunStatus::Success })),
                )
                .into(),
            ])
            .into(),
        ])
        .await;

        // Action! {
        //     { wat ter face}
        // };

        // should be this ??
        // tree! {
        //     <Sequence>
        //         <Action do={some_async_func}/>
        //         <Action do={async {
        //             // an block async
        //         }}/>
        //     </Sequence>
        // }
        //
        // or like this
        // Sequence! {
        //     Action! {
        //         something().await;
        //     }
        //     Action! {
        //         something().await;
        //     }
        //     Action! {
        //         something().await;
        //     }
        //     Decorator! {
        //         conditionFunction,
        //         child will be run,
        //     }
        //     Decorator! {
        //         conditionFunction,
        //         Sequence! {
        //             so on
        //         }
        //     }
        // }
    }
}
