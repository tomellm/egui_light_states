use std::{any::Any, collections::HashMap};

use egui::Ui;

mod other;
pub mod promise_await;
pub mod default_promise_await;
pub mod timer;
pub mod future_await;

pub trait UiWithState {
    fn ui(&mut self, ui: &mut Ui);
}

#[derive(Default)]
pub struct UiStates {
    pub(crate) states: HashMap<String, Box<dyn Any + Send + 'static>>,
}

impl UiStates {
    pub(crate) fn get_mut<'state, StateType>(
        &'state mut self,
        name: String,
        init_state: StateType,
    ) -> &'state mut StateType
    where
        StateType: Send + 'static,
    {
        self.states
            .entry(name)
            .or_insert(Box::new(init_state))
            .downcast_mut::<StateType>()
            .unwrap()
    }
}

pub trait InternalStateTraits
where
    Self: Send + 'static,
{
    fn to(&mut self) -> &mut dyn Any;
}

impl<T> InternalStateTraits for T
where
    T: Send + 'static,
{
    fn to(&mut self) -> &mut dyn Any {
        self
    }
}

pub trait UserStateTraits
where
    Self: InternalStateTraits + Default,
{
}

impl<T> UserStateTraits for T where T: InternalStateTraits + Default {}
