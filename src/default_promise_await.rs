use egui::{Spinner, Ui};
use lazy_async_promise::{ImmediateValuePromise, ImmediateValueState};

use crate::UiStates;

pub trait DefaultCreatePromiseAwait {
    fn default_promise_await<'state, InitUi, PromiseOut>(
        &'state mut self,
        name: String,
    ) -> DefaultPromiseAwaitBuilder<'state, InitUi, PromiseOut>
    where
        InitUi: InitUiTraits<PromiseOut>,
        PromiseOut: Send + 'static;
}

impl DefaultCreatePromiseAwait for UiStates {
    fn default_promise_await<'state, InitUi, PromiseOut>(
        &'state mut self,
        name: String,
    ) -> DefaultPromiseAwaitBuilder<'state, InitUi, PromiseOut>
    where
        InitUi: InitUiTraits<PromiseOut>,
        PromiseOut: Send + 'static,
    {
        let state = self.get_mut(name, DefaultPromiseAwaitState::default());
        DefaultPromiseAwaitBuilder {
            internal_state: state,
            init_ui: None,
        }
    }
}

pub struct DefaultPromiseAwaitBuilder<'state, InitUi, PromiseOut>
where
    InitUi: InitUiTraits<PromiseOut>,
    PromiseOut: Send + 'static,
{
    internal_state: &'state mut DefaultPromiseAwaitState<PromiseOut>,
    init_ui: Option<InitUi>,
}

impl<'state, InitUi, PromiseOut> DefaultPromiseAwaitBuilder<'state, InitUi, PromiseOut>
where
    InitUi: InitUiTraits<PromiseOut>,
    PromiseOut: Send + 'static,
{
    /// init_ui wants a function that takes 
    /// ```rust
    ///     |&mut Ui, &mut dyn FnMut(ImmediateValuePromise<PromiseOut>)|
    /// ```
    /// aka
    /// ```rust
    ///     |ui, set_promise|
    /// ```
    #[must_use]
    pub fn init_ui(mut self, ui: InitUi) -> Self {
        self.init_ui = Some(ui);
        self
    }
    pub fn show(self, ui: &mut Ui) {
        let Self {
            internal_state,
            init_ui: Some(init_ui),
        } = self
        else {
            unreachable!();
        };
        if let DefaultPromiseAwaitState {
            promise: Some(running_promise),
        } = internal_state
        {
            let state = running_promise.poll_state();
            if matches!(state, ImmediateValueState::Updating) {
                ui.add(Spinner::new());
            } else {
                ui.label(match state {
                    ImmediateValueState::Empty => "empty",
                    ImmediateValueState::Success(_) => "success",
                    ImmediateValueState::Error(_) => "error",
                    ImmediateValueState::Updating => unreachable!()
                }.to_string());
                if ui.button("reset").clicked() {
                    internal_state.clear();
                } 
            }
        } else {
            let mut set = |promise| internal_state.set(promise);
            init_ui(ui, &mut set);
        }
    }
}

pub enum DoneResponse {
    KeepShowing,
    Clear,
}

struct DefaultPromiseAwaitState<PromiseOut: Send + 'static> {
    promise: Option<ImmediateValuePromise<PromiseOut>>,
}

impl<PromiseOut: Send + 'static> Default for DefaultPromiseAwaitState<PromiseOut> {
    fn default() -> Self {
        Self { promise: None }
    }
}

impl<PromiseOut: Send + 'static> DefaultPromiseAwaitState<PromiseOut> {
    fn set(&mut self, promise: ImmediateValuePromise<PromiseOut>) {
        self.promise = Some(promise);
    }
    fn clear(&mut self) {
        self.promise = None;
    }
}

pub trait InitUiTraits<PromiseOut>
where
    Self: FnOnce(&mut Ui, &mut dyn FnMut(ImmediateValuePromise<PromiseOut>)),
    PromiseOut: Send + 'static,
{
}

impl<T, PromiseOut> InitUiTraits<PromiseOut> for T
where
    T: FnOnce(&mut Ui, &mut dyn FnMut(ImmediateValuePromise<PromiseOut>)),
    PromiseOut: Send + 'static,
{
}
