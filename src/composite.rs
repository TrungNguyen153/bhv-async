use std::{future::Future, pin::Pin, rc::Rc};

use crate::RunStatus;

/// Can create from Box::pin(an future)
pub type BoxAction = Pin<Box<dyn Future<Output = RunStatus>>>;

#[derive(Clone)]
pub struct Composite {
    pub name: String,
    // Box not allow clone
    // Rc will hold data and share it for clone
    pub task_production: Rc<dyn Fn() -> BoxAction>,
}

/// Require:
/// - Clone
/// - Became future -> RunStatus
#[macro_export]
macro_rules! IMPLEMENT_INTO_COMPOSITE {
    ($type:ty) => {
        impl From<$type> for Composite {
            fn from(value: $type) -> Self {
                $crate::composite::Composite::new(stringify!($type), move || {
                    let value_go = value.clone();
                    Box::pin(value_go)
                })
            }
        }
    };
}

impl Composite {
    pub fn new(name: impl Into<String>, task_production: impl Fn() -> BoxAction + 'static) -> Self {
        let name = name.into();
        Self {
            name,
            task_production: Rc::new(task_production),
        }
    }

    pub fn new_action(task_production: impl Fn() -> BoxAction + 'static) -> Self {
        Self {
            name: "Action".into(),
            task_production: Rc::new(task_production),
        }
    }
}
