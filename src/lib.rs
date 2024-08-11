#![feature(unboxed_closures)]

use std::{any::Any, collections::HashMap};

use egui::Ui;

mod other;
pub mod promise_await;
pub mod timer;

pub trait UiWithState {
    fn ui(&mut self, ui: &mut Ui);
}

#[derive(Default)]
pub struct UiStates {
    pub(crate) states: HashMap<&'static str, Box<dyn Any>>,
}

impl UiStates {
    pub(crate) fn get_mut<'state, StateType>(
        &'state mut self,
        name: &'static str,
        init_state: StateType,
    ) -> &'state mut StateType
    where
        StateType: 'static,
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
    Self: 'static,
{
    fn to(&mut self) -> &mut dyn Any;
}

impl<T> InternalStateTraits for T
where
    T: 'static,
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
