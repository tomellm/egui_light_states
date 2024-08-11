use egui::{Spinner, Ui};
use lazy_async_promise::{ImmediateValuePromise, ImmediateValueState};

use crate::UiStates;

pub trait CreatePromiseAwait {
    fn promise_await<'state, InitUi, WaitingUi, DoneUi, PromiseOut>(
        &'state mut self,
        name: &'static str,
    ) -> PromiseAwaitBuilder<'state, InitUi, WaitingUi, DoneUi, PromiseOut>
    where
        InitUi: InitUiTraits<PromiseOut>,
        WaitingUi: WaitingUiTraits,
        DoneUi: DoneUiTraits<PromiseOut>,
        PromiseOut: Send + 'static;
}

impl CreatePromiseAwait for UiStates {
    fn promise_await<'state, InitUi, WaitingUi, DoneUi, PromiseOut>(
        &'state mut self,
        name: &'static str,
    ) -> PromiseAwaitBuilder<'state, InitUi, WaitingUi, DoneUi, PromiseOut>
    where
        InitUi: InitUiTraits<PromiseOut>,
        WaitingUi: WaitingUiTraits,
        DoneUi: DoneUiTraits<PromiseOut>,
        PromiseOut: Send + 'static,
    {
        let state = self.get_mut(name, PromiseAwaitState::default());
        PromiseAwaitBuilder {
            internal_state: state,
            init_ui: None,
            waiting_ui: None,
            done_ui: None,
        }
    }
}

pub struct PromiseAwaitBuilder<'state, InitUi, WaitingUi, DoneUi, PromiseOut>
where
    InitUi: InitUiTraits<PromiseOut>,
    WaitingUi: WaitingUiTraits,
    DoneUi: DoneUiTraits<PromiseOut>,
    PromiseOut: Send + 'static,
{
    internal_state: &'state mut PromiseAwaitState<PromiseOut>,
    init_ui: Option<InitUi>,
    waiting_ui: Option<WaitingUi>,
    done_ui: Option<DoneUi>,
}

impl<'state, InitUi, WaitingUi, DoneUi, PromiseOut>
    PromiseAwaitBuilder<'state, InitUi, WaitingUi, DoneUi, PromiseOut>
where
    InitUi: InitUiTraits<PromiseOut>,
    WaitingUi: WaitingUiTraits,
    DoneUi: DoneUiTraits<PromiseOut>,
    PromiseOut: Send + 'static,
{
    pub fn init_ui(mut self, ui: InitUi) -> Self {
        self.init_ui = Some(ui);
        self
    }
    pub fn waiting_ui(mut self, ui: WaitingUi) -> Self {
        self.waiting_ui = Some(ui);
        self
    }
    pub fn done_ui(mut self, ui: DoneUi) -> Self {
        self.done_ui = Some(ui);
        self
    }
    pub fn show(self, ui: &mut Ui) {
        let Self {
            internal_state,
            init_ui: Some(init_ui),
            waiting_ui: Some(waiting_ui),
            done_ui: Some(done_ui),
        } = self
        else {
            unreachable!();
        };
        if let PromiseAwaitState {
            promise: Some(running_promise),
        } = internal_state
        {
            let state = running_promise.poll_state();
            if matches!(state, ImmediateValueState::Updating) {
                waiting_ui(ui);
            } else {
                let ui_response = done_ui(ui, state);
                if matches!(ui_response, DoneResponse::Clear) {
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

struct PromiseAwaitState<PromiseOut: Send + 'static> {
    promise: Option<ImmediateValuePromise<PromiseOut>>,
}

impl<PromiseOut: Send + 'static> Default for PromiseAwaitState<PromiseOut> {
    fn default() -> Self {
        Self { promise: None }
    }
}

impl<PromiseOut: Send + 'static> PromiseAwaitState<PromiseOut> {
    fn set(&mut self, promise: ImmediateValuePromise<PromiseOut>) {
        self.promise = Some(promise);
    }
    fn clear(&mut self) {
        self.promise = None;
    }
}

pub trait InitUiTraits<PromiseOut>
where
    Self: FnOnce(&mut Ui, &mut dyn FnMut(ImmediateValuePromise<PromiseOut>) -> ()),
    PromiseOut: Send + 'static,
{
}

impl<T, PromiseOut> InitUiTraits<PromiseOut> for T
where
    T: FnOnce(&mut Ui, &mut dyn FnMut(ImmediateValuePromise<PromiseOut>) -> ()),
    PromiseOut: Send + 'static,
{
}

pub trait WaitingUiTraits
where
    Self: FnOnce(&mut Ui),
{
}

impl<T> WaitingUiTraits for T where T: FnOnce(&mut Ui) {}

pub trait DoneUiTraits<PromiseOut>
where
    Self: FnOnce(&mut Ui, &ImmediateValueState<PromiseOut>) -> DoneResponse,
{
}

impl<T, PromiseOut> DoneUiTraits<PromiseOut> for T where
    T: FnOnce(&mut Ui, &ImmediateValueState<PromiseOut>) -> DoneResponse
{
}
