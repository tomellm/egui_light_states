use chrono::{DateTime, Duration, Local};
use egui::Ui;

use crate::{UiStates, UserStateTraits};

pub trait CreateTimerUi {
    fn timer<'state, DoneUi, TimingUi, UserState>(
        &'state mut self,
        name: String,
        duration: i64,
    ) -> TimerBuilder<'state, DoneUi, TimingUi, UserState>
    where
        DoneUi: DoneUiTraits<UserState>,
        TimingUi: TimingUiTraits<UserState>,
        UserState: UserStateTraits;
}

impl CreateTimerUi for UiStates {
    fn timer<'state, DoneUi, TimingUi, UserState>(
        &'state mut self,
        name: String,
        duration: i64,
    ) -> TimerBuilder<'state, DoneUi, TimingUi, UserState>
    where
        DoneUi: DoneUiTraits<UserState>,
        TimingUi: TimingUiTraits<UserState>,
        UserState: UserStateTraits,
    {

        let CompleteTimerState::<UserState> {
            internal_state,
            user_state,
        } = self.get_mut(name, CompleteTimerState::from(TimerState::from(duration)));
        TimerBuilder {
            internal_state,
            user_state,
            timer_done_ui: None,
            timer_timing_ui: None,
        }
    }
}

pub struct TimerBuilder<'state, DoneUi, TimingUi, UserState>
where
    DoneUi: DoneUiTraits<UserState>,
    TimingUi: TimingUiTraits<UserState>,
    UserState: UserStateTraits,
{
    internal_state: &'state mut TimerState,
    user_state: &'state mut UserState,
    timer_done_ui: Option<DoneUi>,
    timer_timing_ui: Option<TimingUi>,
}

impl<'state, DoneUi, TimingUi, State> TimerBuilder<'state, DoneUi, TimingUi, State>
where
    DoneUi: DoneUiTraits<State>,
    TimingUi: TimingUiTraits<State>,
    State: UserStateTraits,
{
    pub fn timer_done_ui(mut self, ui: DoneUi) -> Self {
        self.timer_done_ui = Some(ui);
        self
    }
    pub fn timing_ui(mut self, ui: TimingUi) -> Self {
        self.timer_timing_ui = Some(ui);
        self
    }
    pub fn show(self, ui: &mut Ui) {
        let TimerBuilder {
            internal_state,
            user_state,
            timer_done_ui: Some(timer_done_ui),
            timer_timing_ui: Some(timer_timing_ui),
        } = self
        else {
            unreachable!()
        };
        let TimerState {
            timer_started,
            timer_duration,
        } = internal_state;
        match timer_started {
            None => {
                let mut reset_timer = || {
                    let _ = timer_started.insert(Local::now());
                };
                timer_done_ui(ui, user_state, &mut reset_timer);
            }
            Some(start_time) => {
                let now = Local::now();
                let passed_time = now - *start_time;

                let percentage_passed = passed_time.num_milliseconds() as f64
                    / timer_duration.num_milliseconds() as f64;

                timer_timing_ui(ui, user_state, percentage_passed as f32);

                if passed_time >= *timer_duration {
                    let _ = timer_started.take();
                }
            }
        }
    }
}

struct CompleteTimerState<UserState>
where
    UserState: UserStateTraits
{
    internal_state: TimerState,
    user_state: UserState,
}

impl<UserState> From<TimerState> for CompleteTimerState<UserState>
where
    UserState: UserStateTraits
{
    fn from(value: TimerState) -> Self {
        Self { internal_state: value, user_state: UserState::default() }
    }
}

pub struct TimerState {
    timer_started: Option<DateTime<Local>>,
    timer_duration: Duration,
}

impl From<i64> for TimerState {
    fn from(value: i64) -> Self {
        Self {
            timer_started: None,
            timer_duration: Duration::seconds(value),
        }
    }
}

pub trait DoneUiTraits<State>
where
    Self: FnOnce(&mut Ui, &mut State, &mut dyn FnMut()) + 'static,
    State: UserStateTraits,
{
}

impl<State, T> DoneUiTraits<State> for T
where
    T: FnOnce(&mut Ui, &mut State, &mut dyn FnMut()) + 'static,
    State: UserStateTraits,
{
}

pub trait TimingUiTraits<State>
where
    Self: FnOnce(&mut Ui, &mut State, f32) + 'static,
    State: UserStateTraits,
{
}

impl<State, T> TimingUiTraits<State> for T
where
    T: FnOnce(&mut Ui, &mut State, f32) + 'static,
    State: UserStateTraits,
{
}
