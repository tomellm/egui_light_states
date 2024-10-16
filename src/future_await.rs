use egui::{Spinner, Ui};
use lazy_async_promise::{
    BoxedSendError, DirectCacheAccess, ImmediateValuePromise, ImmediateValueState,
};

use crate::UiStates;

pub trait FutureAwait {
    fn is_running<'state, T>(&'state mut self, name: impl Into<String>) -> bool
    where
        T: Send + 'static;
    #[must_use]
    fn set_future<'state, T>(
        &'state mut self,
        name: impl Into<String>,
    ) -> SetFutureBuilder<'state, T>
    where
        T: Send + 'static;
    #[must_use]
    fn future_status<'state, T>(
        &'state mut self,
        name: impl Into<String>,
    ) -> FutureStatusBuilder<'state, T>
    where
        T: Send + 'static;
}

impl FutureAwait for UiStates {
    fn is_running<'state, T>(&'state mut self, name: impl Into<String>) -> bool
    where
        T: Send + 'static,
    {
        self.get_mut::<Option<ImmediateValuePromise<T>>>(name.into(), None)
            .as_mut()
            .map(|promise| matches!(promise.poll_state(), ImmediateValueState::Updating))
            .unwrap_or(false)
    }
    #[must_use]
    fn set_future<'state, T>(
        &'state mut self,
        name: impl Into<String>,
    ) -> SetFutureBuilder<'state, T>
    where
        T: Send + 'static,
    {
        let state = self.get_mut(name.into(), None);
        SetFutureBuilder { state }
    }
    #[must_use]
    fn future_status<'state, T>(
        &'state mut self,
        name: impl Into<String>,
    ) -> FutureStatusBuilder<'state, T>
    where
        T: Send + 'static,
    {
        let state = self.get_mut(name.into(), None);
        FutureStatusBuilder {
            state,
            waiting_ui: None,
            empty_ui: None,
            done_ui: None,
        }
    }
}

pub struct SetFutureBuilder<'state, T>
where
    T: Send + 'static,
{
    state: &'state mut Option<ImmediateValuePromise<T>>,
}

impl<'state, T> SetFutureBuilder<'state, T>
where
    T: Send + 'static,
{
    pub fn set(self, future: impl Into<ImmediateValuePromise<T>>) {
        *self.state = Some(future.into());
    }
}

pub struct FutureStatusBuilder<'state, T>
where
    T: Send + 'static,
{
    state: &'state mut Option<ImmediateValuePromise<T>>,
    waiting_ui: Option<Box<dyn FnOnce(&mut Ui)>>,
    empty_ui: Option<Box<dyn FnOnce(&mut Ui)>>,
    done_ui: Option<Box<dyn FnOnce(&mut Ui, Result<&T, &BoxedSendError>, &mut dyn FnMut())>>,
}

impl<'state, T> FutureStatusBuilder<'state, T>
where
    T: Send + 'static,
{
    #[must_use]
    pub fn default(self) -> Self {
        self.spinner()
            .empty_ui(|_| {})
            .done_ui(|ui, result, reset| {
                match result {
                    Ok(_) => ui.label("success"),
                    Err(_) => ui.label("error"),
                };
                if ui.button("clear").clicked() {
                    reset();
                }
            })
    }
    #[must_use]
    pub fn done_ui(
        mut self,
        done_ui: impl FnOnce(&mut Ui, Result<&T, &BoxedSendError>, &mut dyn FnMut()) + 'static,
    ) -> Self {
        self.done_ui = Some(Box::new(done_ui));
        self
    }
    #[must_use]
    pub fn empty_ui(mut self, empty_ui: impl FnOnce(&mut Ui) + 'static) -> Self {
        self.empty_ui = Some(Box::new(empty_ui));
        self
    }
    #[must_use]
    pub fn spinner(mut self) -> Self {
        self.waiting_ui = Some(Box::new(|ui| {
            ui.add(Spinner::new());
        }));
        self
    }
    pub fn only_poll(self) {
        self.state.as_mut().map(|promise| {
            promise.poll_state();
        });
    }
    pub fn show(self, ui: &mut Ui) {
        let Some(promise) = self.state else {
            self.empty_ui.map(|empty_ui| {
                empty_ui(ui);
            });
            return;
        };
        if matches!(promise.poll_state(), ImmediateValueState::<T>::Updating) {
            self.waiting_ui.map(|waiting_ui| {
                waiting_ui(ui);
            });
        } else {
            let mut reset = false;
            match promise.get_result() {
                Some(result) => {
                    self.done_ui.map(|done_ui| {
                        let mut reset_fn = || reset = true;
                        done_ui(ui, result, &mut reset_fn);
                    });
                }
                None => {
                    *self.state = None;
                }
            }
            if reset {
                *self.state = None;
            }
        }
    }
}
