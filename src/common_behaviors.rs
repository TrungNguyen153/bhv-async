use std::{future::Future, pin::Pin, rc::Rc, task::Poll};

use crate::{
    composite::{BoxAction, Composite},
    RunStatus,
};

/// An group action execute each branch of logic, in order.
/// If all branches succeed, this composite will return a successful run status.
/// If any branch fails, this composite will return a failed run status.
#[derive(Default)]
pub struct Sequence {
    childs: Vec<Composite>,
    index: usize,
    fut: Option<BoxAction>,
}

impl Clone for Sequence {
    fn clone(&self) -> Self {
        Self {
            childs: self.childs.clone(),
            ..Default::default()
        }
    }
}

IMPLEMENT_INTO_COMPOSITE!(Sequence);

impl Sequence {
    pub fn new(childs: impl Into<Vec<Composite>>) -> Self {
        Self {
            childs: childs.into(),
            ..Default::default()
        }
    }
}

/// Those children composite will not break composite Selector
/// Composite Selector: PrioritySelector,...
/// maybe export api for add more selector
/// => maybe should use "static mut"
const OPTIONAL_CHILD_NAMES: [&str; 1] = ["DecoratorContinue"];

impl Future for Sequence {
    type Output = RunStatus;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { Pin::into_inner_unchecked(self) };
        if this.childs.is_empty() {
            return Poll::Ready(RunStatus::Success);
        }

        if this.fut.is_none() {
            let index = this.index;
            let child = &this.childs[index];
            println!(
                "Running task {} ({}/{})",
                child.name,
                index + 1,
                this.childs.len()
            );
            let fut = (child.task_production)();
            this.fut = Some(fut);
        }

        // let fut = this.fut.as_mut().unwrap();
        match Pin::new(this.fut.as_mut().unwrap()).poll(cx) {
            Poll::Ready(status) => {
                if status == RunStatus::Failure {
                    return Poll::Ready(RunStatus::Failure);
                }
                if this.index + 1 >= this.childs.len() {
                    return Poll::Ready(RunStatus::Success);
                }
                this.index += 1;
                this.fut.take();

                // NOTE
                // When context poll done an task. it will no longer getting poll
                // we need notifiy executor we have more task need scheduling.
                // call wake_by_ref will make it work.
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// An Selector execute each branch of logic in order, until one succeeds. This composite
/// will fail only if all branches fail as well.
///
/// This composite type is Selector
#[derive(Default)]
pub struct PrioritySelector {
    childs: Vec<Composite>,
    index: usize,
    is_running_optional_child: bool,
    fut: Option<BoxAction>,
}

impl Clone for PrioritySelector {
    fn clone(&self) -> Self {
        Self {
            childs: self.childs.clone(),
            ..Default::default()
        }
    }
}

impl PrioritySelector {
    pub fn new(childs: impl Into<Vec<Composite>>) -> Self {
        Self {
            childs: childs.into(),
            ..Default::default()
        }
    }
}

impl Future for PrioritySelector {
    type Output = RunStatus;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { Pin::into_inner_unchecked(self) };
        if this.childs.is_empty() {
            return Poll::Ready(RunStatus::Failure);
        }

        if this.fut.is_none() {
            let index = this.index;
            let child = &this.childs[index];
            println!(
                "Running task {} ({}/{})",
                child.name,
                index + 1,
                this.childs.len()
            );
            let fut = (child.task_production)();
            this.fut = Some(fut);
            this.is_running_optional_child = OPTIONAL_CHILD_NAMES.contains(&&*child.name);
        }

        // let fut = this.fut.as_mut().unwrap();

        match Pin::new(this.fut.as_mut().unwrap()).poll(cx) {
            Poll::Ready(mut status) => {
                if this.is_running_optional_child {
                    // overwrite status to failure if running Optional child
                    status = RunStatus::Failure;
                }

                if status == RunStatus::Success {
                    return Poll::Ready(RunStatus::Success);
                }
                if status == RunStatus::Failure && this.index + 1 >= this.childs.len() {
                    return Poll::Ready(RunStatus::Failure);
                }
                this.index += 1;
                this.fut.take();

                // NOTE
                // When context poll done an task. it will no longer getting poll
                // we need notifiy executor we have more task need scheduling.
                // call wake_by_ref will make it work.
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

IMPLEMENT_INTO_COMPOSITE!(PrioritySelector);

/// A decorator that allows you to execute code only if some condition is met.
/// Otherwise, return failed.
pub struct Decorator {
    run_condition: Rc<dyn Fn() -> bool>,
    child: Composite,
    fut: Option<BoxAction>,
}
impl Clone for Decorator {
    fn clone(&self) -> Self {
        Self {
            run_condition: self.run_condition.clone(),
            child: self.child.clone(),
            fut: None,
        }
    }
}

impl Decorator {
    pub fn new(condition: impl Fn() -> bool + 'static, child: impl Into<Composite>) -> Self {
        Self {
            run_condition: Rc::new(condition),
            child: child.into(),
            fut: None,
        }
    }
}

impl Future for Decorator {
    type Output = RunStatus;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        if !(self.run_condition)() {
            return Poll::Ready(RunStatus::Failure);
        }

        if self.fut.is_none() {
            let fut = (self.child.task_production)();
            self.fut = Some(fut);
        }
        Pin::new(self.fut.as_mut().unwrap()).poll(cx)
    }
}

IMPLEMENT_INTO_COMPOSITE!(Decorator);

/// A decorator that allows you to execute code only if some condition is met. It does not 'break' the current
/// tree if the CONDITION FAILS, or CHILDREN FAIL.
///
/// This is useful for "if I need to, go ahead, otherwise, ignore" in sequences.
///
/// It can be thought of as an optional execution.
pub struct DecoratorContinue {
    run_condition: Rc<dyn Fn() -> bool>,
    child: Composite,
    fut: Option<BoxAction>,
}

impl Clone for DecoratorContinue {
    fn clone(&self) -> Self {
        Self {
            run_condition: self.run_condition.clone(),
            child: self.child.clone(),
            fut: None,
        }
    }
}

impl DecoratorContinue {
    pub fn new(condition: impl Fn() -> bool + 'static, child: impl Into<Composite>) -> Self {
        Self {
            run_condition: Rc::new(condition),
            child: child.into(),
            fut: None,
        }
    }
}
impl Future for DecoratorContinue {
    type Output = RunStatus;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        if !(self.run_condition)() {
            return Poll::Ready(RunStatus::Success);
        }

        if self.fut.is_none() {
            let fut = (self.child.task_production)();
            self.fut = Some(fut);
        }
        match Pin::new(self.fut.as_mut().unwrap()).poll(cx) {
            Poll::Ready(_) => Poll::Ready(RunStatus::Success),
            Poll::Pending => Poll::Pending,
        }
    }
}

IMPLEMENT_INTO_COMPOSITE!(DecoratorContinue);

/// An action running with check condition between Poll
/// if condition not met.. it will return Success immediately (stop action with success status)
/// Otherwise, it will return action status after finish
pub struct InterruptAction {
    run_condition: Rc<dyn Fn() -> bool>,
    child: Composite,
    fut: Option<BoxAction>,
}

impl Clone for InterruptAction {
    fn clone(&self) -> Self {
        Self {
            run_condition: self.run_condition.clone(),
            child: self.child.clone(),
            fut: None,
        }
    }
}

impl InterruptAction {
    pub fn new(condition: impl Fn() -> bool + 'static, child: impl Into<Composite>) -> Self {
        Self {
            run_condition: Rc::new(condition),
            child: child.into(),
            fut: None,
        }
    }
}

impl Future for InterruptAction {
    type Output = RunStatus;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let fut = (self.child.task_production)();
            self.fut = Some(fut);
        }
        match Pin::new(self.fut.as_mut().unwrap()).poll(cx) {
            Poll::Ready(status) => Poll::Ready(status),
            Poll::Pending => {
                if !(self.run_condition)() {
                    println!("Trigger interrupt");
                    return Poll::Ready(RunStatus::Failure);
                }
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

IMPLEMENT_INTO_COMPOSITE!(InterruptAction);

/// Run action then return status Inverter
/// Success became Failed
/// Failed became Success
pub struct Inverter {
    child: Composite,
    fut: Option<BoxAction>,
}

impl Clone for Inverter {
    fn clone(&self) -> Self {
        Self {
            child: self.child.clone(),
            fut: None,
        }
    }
}

impl Inverter {
    pub fn new(child: impl Into<Composite>) -> Self {
        Self {
            child: child.into(),
            fut: None,
        }
    }
}

impl Future for Inverter {
    type Output = RunStatus;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let fut = (self.child.task_production)();
            self.fut = Some(fut);
        }
        match Pin::new(self.fut.as_mut().unwrap()).poll(cx) {
            Poll::Ready(status) => match status {
                RunStatus::Success => Poll::Ready(RunStatus::Failure),
                RunStatus::Failure => Poll::Ready(RunStatus::Success),
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

IMPLEMENT_INTO_COMPOSITE!(Inverter);

/// Run action until it success
/// Mean ignore failure, recreate and run again until success
pub struct UntilSuccess {
    child: Composite,
    fut: Option<BoxAction>,
}

impl Clone for UntilSuccess {
    fn clone(&self) -> Self {
        Self {
            child: self.child.clone(),
            fut: None,
        }
    }
}

impl UntilSuccess {
    pub fn new(child: impl Into<Composite>) -> Self {
        Self {
            child: child.into(),
            fut: None,
        }
    }
}

impl Future for UntilSuccess {
    type Output = RunStatus;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let fut = (self.child.task_production)();
            self.fut = Some(fut);
        }
        match Pin::new(self.fut.as_mut().unwrap()).poll(cx) {
            Poll::Ready(status) => {
                if status == RunStatus::Failure {
                    self.fut.take();
                    cx.waker().wake_by_ref();
                    Poll::Pending
                } else {
                    Poll::Ready(status)
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

IMPLEMENT_INTO_COMPOSITE!(UntilSuccess);

/// Run action until it failure
/// Mean ignore success, recreate and run again until failure
pub struct UntilFailure {
    child: Composite,
    fut: Option<BoxAction>,
}
impl Clone for UntilFailure {
    fn clone(&self) -> Self {
        Self {
            child: self.child.clone(),
            fut: None,
        }
    }
}

impl UntilFailure {
    pub fn new(child: impl Into<Composite>) -> Self {
        Self {
            child: child.into(),
            fut: None,
        }
    }
}
impl Future for UntilFailure {
    type Output = RunStatus;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let fut = (self.child.task_production)();
            self.fut = Some(fut);
        }
        match Pin::new(self.fut.as_mut().unwrap()).poll(cx) {
            Poll::Ready(status) => {
                if status == RunStatus::Success {
                    self.fut.take();
                    cx.waker().wake_by_ref();
                    Poll::Pending
                } else {
                    Poll::Ready(status)
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

IMPLEMENT_INTO_COMPOSITE!(UntilFailure);

// TODO:
// WhileNode
// IfNode
