//! Check out [`UiStates`] to get started with this crate, after which come back
//! here.
//!
//! The [`FutureAwait`] trait provides three functions to store and then view
//! some kind of async state.
//!
//! Stores future in internal state through [`set_future`][set_fut] and then provides
//! methods to acceess state of internal future like: [`is_running`][is_run] and
//! [`future_status`][fut_stat]
//!
//! ```
//! if !self.ui.is_running::<T>("future_name")
//!     && ui.button("Save parsed Data").clicked()
//! {
//!     let future = self.save_parsed_data();
//!     self.ui.set_future("future_name").set(future);
//! }
//! self.ui
//!     .future_status::<()>("future_name")
//!     .default()
//!     .show(ui);
//! ```
//!
//! [is_run]: FutureAwait::is_running
//! [set_fut]: FutureAwait::set_future
//! [fut_stat]: FutureAwait::future_status

use egui::{Spinner, Ui};
use lazy_async_promise::{
    BoxedSendError, DirectCacheAccess, ImmediateValuePromise, ImmediateValueState,
};

use crate::UiStates;

/// Stores future in internal state through [`set_future`][FutureAwait::set_future] and then provides
/// methods to acceess state of internal future.
///
/// ```
/// if !self.ui.is_running::<T>("future_name")
///     && ui.button("Save parsed Data").clicked()
/// {
///     let future = self.save_parsed_data();
///     self.ui.set_future("future_name").set(future);
/// }
/// self.ui
///     .future_status::<()>("future_name")
///     .default()
///     .show(ui);
/// ```
pub trait FutureAwait {
    /// Part of trait [`FutureAwait`] To be used in combination with the functions: [`set_future`][set_fut]
    /// and/or [`future_status`][fut_stat]
    ///
    /// Will poll the internal future updating its state and checking if its still
    /// running. Useful because it allows you to poll the future without having
    /// to use [`future_status`][fut_stat]
    ///
    /// ```
    /// let ui_state: UiStates;
    ///
    /// // -- your code
    ///
    /// if !ui_state.is_running::<T>("future_name")
    ///     && ui.button("Save Data").clicked()
    /// {
    ///     let future = // -- some function returning a future
    ///     ui_state.set_future("future_name").set(future);
    /// }
    /// ```
    ///
    /// [set_fut]: crate::future_await::FutureAwait::set_future
    /// [fut_stat]: crate::future_await::FutureAwait::future_status
    fn is_running<T>(&mut self, name: impl Into<String>) -> bool
    where
        T: Send + 'static;

    /// Part of trait [`FutureAwait`]
    /// To be used in combination with the functions: [`is_running`][is_run]
    /// and/or [`future_status`][fut_stat]
    ///
    /// Will set the passed future as the internal state of the defined name.
    ///
    /// ```
    /// self.ui.set_future("state_name").set(future);
    /// ```
    ///
    /// [is_run]: crate::future_await::FutureAwait::is_running
    /// [fut_stat]: crate::future_await::FutureAwait::future_status
    #[must_use]
    fn set_future<T>(&mut self, name: impl Into<String>) -> SetFutureBuilder<T>
    where
        T: Send + 'static;

    /// Part of trait [`FutureAwait`]
    /// To be used in combination with the functions: [`is_running`][is_run]
    /// and/or [`set_future`][set_fut]
    ///
    /// Allows for viewing and modifying of the internal future state from different
    /// locations.
    ///
    /// ```
    /// self.state
    ///     .future_status::<T>("your_state_name")
    ///     .default()
    ///     .show(ui);
    /// ```
    ///
    /// Check out the [`FutureStatusBuilder`] for all of the customization
    /// options for the status.
    ///
    /// [is_run]: crate::future_await::FutureAwait::is_running
    /// [set_fut]: crate::future_await::FutureAwait::set_future
    #[must_use]
    fn future_status<T>(&mut self, name: impl Into<String>) -> FutureStatusBuilder<T>
    where
        T: Send + 'static;
}

impl FutureAwait for UiStates {
    fn is_running<T>(&mut self, name: impl Into<String>) -> bool
    where
        T: Send + 'static,
    {
        self.get_mut::<Option<ImmediateValuePromise<T>>>(name.into(), None)
            .as_mut()
            .map(|promise| matches!(promise.poll_state(), ImmediateValueState::Updating))
            .unwrap_or(false)
    }
    #[must_use]
    fn set_future<T>(&mut self, name: impl Into<String>) -> SetFutureBuilder<T>
    where
        T: Send + 'static,
    {
        let state = self.get_mut(name.into(), None);
        SetFutureBuilder { state }
    }
    #[must_use]
    fn future_status<T>(&mut self, name: impl Into<String>) -> FutureStatusBuilder<T>
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
