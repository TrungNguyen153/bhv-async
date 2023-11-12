use std::time::Duration;

use bhv_async::prelude::*;
use tokio::time::sleep;

fn main() {
    let tree = create_tree();
    test_run_in_sync_fn(tree.clone());
    test_run_in_async_fn(tree.clone());
    println!("Done");
}

fn create_tree() -> Composite {
    let action = Action! {
        "Bushit",
        || async {
            sleep(Duration::from_secs(1)).await;
            RunStatus::Success
        }
    };

    // capture will cant known name, unless you mark name for it
    let _action_with_capture = Action! {
        "Bushit",
        move || {
            let action_move = action.clone();
            return async move {
                (action_move.task_production)().await;
                RunStatus::Success
            }
        }
    };

    struct PrintOnDrop(String);
    impl Drop for PrintOnDrop {
        fn drop(&mut self) {
            println!("{}", self.0);
        }
    }

    Sequence! {
        _action_with_capture,
        Action! {
            "FirstChildinSequence",
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
                let _a = PrintOnDrop("Cleanup 0".into());
                sleep(Duration::from_secs(1)).await;
                RunStatus::Failure
            }},
            DecoratorContinue! {
                || true,
                || async {
                println!("Should run 5");
                let _a = PrintOnDrop("Cleanup 1".into());
                sleep(Duration::from_secs(1)).await;
                RunStatus::Failure
            }},
            Decorator! {
                || true,
                || async {
                println!("Should run 4");
                let _a = PrintOnDrop("Cleanup 2".into());
                sleep(Duration::from_secs(1)).await;
                println!("After Long Sleep");
                RunStatus::Failure
            }},
        },
    }
    .into()
}

fn test_run_in_sync_fn(composite: Composite) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    let mut task = (composite.task_production)();
    let begin = std::time::Instant::now();
    loop {
        let fut = async {
            std::future::poll_fn(|cx| match task.as_mut().poll(cx) {
                std::task::Poll::Ready(_) => std::task::Poll::Ready(true),
                std::task::Poll::Pending => std::task::Poll::Ready(false),
            })
            .await
        };
        if tokio::runtime::Handle::current().block_on(fut) {
            break;
        }
    }

    let dur = std::time::Instant::now() - begin;
    println!("TEST COST RUN IN SYNC FUNCTION: {dur:?}")
}

fn test_run_in_async_fn(composite: Composite) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut task_created = (composite.task_production)();
    let task = task_created.as_mut();

    rt.block_on(async {
        let begin = std::time::Instant::now();
        task.await;
        let dur = std::time::Instant::now() - begin;
        println!("TEST COST RUN IN ASYNC FUNCTION: {dur:?}")
    });
}
