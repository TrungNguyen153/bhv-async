use std::time::Duration;

use bhv_async::prelude::*;
use tokio::time::sleep;
extern crate bhv_async_macros;

#[tokio::main]
async fn main() {
    let action = Action! {
        || async {
            sleep(Duration::from_secs(1)).await;
            RunStatus::Success
        }
    };

    let _action_with_capture = Action! {
        move || {
            let action_move = action.clone();
            return async move {
                (action_move.task_production)().await;
                RunStatus::Success
            }
        }
    };

    let seq = Sequence! {
        _action_with_capture,
        Action! {
            || async {
                sleep(Duration::from_secs(1)).await;
                RunStatus::Success
            }
        },
        Action! {
            || async {
                sleep(Duration::from_secs(1)).await;
                RunStatus::Success
            }
        },
        DecoratorContinue! {
            || false,
            || async {
            println!("Should not run 1");
            sleep(Duration::from_secs(1)).await;
            RunStatus::Failure
        }},
        Decorator! {
            || true,
            || async {
            println!("Should run 2");
            sleep(Duration::from_secs(1)).await;
            RunStatus::Success
        }},
        PrioritySelector! {
            Action! {
                || async {
                    sleep(Duration::from_secs(1)).await;
                    RunStatus::Failure
                }
            },
            DecoratorContinue! {
                || false,
                || async {
                println!("Should not run 3");
                sleep(Duration::from_secs(1)).await;
                RunStatus::Failure
            }},
            Decorator! {
                || true,
                || async {
                println!("Should run 4");
                sleep(Duration::from_secs(1)).await;
                RunStatus::Failure
            }},

        },
    };
    seq.await;

    // let decor = Decorator! {|| true, decor};
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
    //
    // Action! {
    // || {
    // capture something
    // then ret async
    // async {
    //
    // }
    //
    // }
    // }
}
