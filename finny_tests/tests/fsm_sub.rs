extern crate finny;

use finny::{
    bundled::derive_more, finny_fsm, inspect::slog::InspectSlog, FsmCurrentState, FsmError,
    FsmEventQueueVec, FsmFactory, FsmResult, FsmTimersNull,
};
use slog::{o, Drain};

#[derive(Default)]
pub struct MainContext {
    sub_enter: usize,
    sub_exit: usize,
    sub_action: usize,
}

#[derive(Default)]
pub struct StateA {
    value: usize,
}

#[derive(Debug, Clone)]
pub struct Event;
#[derive(Debug, Clone)]
pub struct EventSub {
    n: usize,
}
#[derive(Debug, Clone)]
pub struct EventSubSecond;

#[finny_fsm]
fn build_fsm(mut fsm: FsmBuilder<StateMachine, MainContext>) -> BuiltFsm {
    fsm.initial_state::<StateA>();
    fsm.state::<StateA>()
        .on_event::<Event>()
        .transition_to::<SubStateMachine>();

    fsm.sub_machine::<SubStateMachine>()
        .with_context(|ctx| SubContext {
            value: ctx.sub_enter,
        })
        .on_entry(|_sub, ctx| {
            ctx.sub_enter += 1;
        })
        .on_exit(|_sub, ctx| {
            ctx.sub_exit += 1;
        })
        .on_event::<Event>()
        .transition_to::<StateA>()
        .action(|_ev, _ctx, _from, to| {
            to.value += 1;
        });

    fsm.sub_machine::<SubStateMachine>()
        .on_event::<EventSub>()
        .self_transition()
        .guard(|ev, _ctx, _| ev.n > 0)
        .action(|_ev, ctx, _state| {
            ctx.context.sub_action += 1;
        });

    fsm.sub_machine::<SubStateMachine>()
        .on_event::<EventSubSecond>()
        .transition_to::<SecondSubStateMachine>();

    fsm.sub_machine::<SecondSubStateMachine>()
        .with_context(|_| SecondSubContext { value: 0 });

    fsm.build()
}

#[derive(Default)]
pub struct SubStateA {
    value: usize,
}
#[derive(Default)]
pub struct SubStateB {
    #[allow(dead_code)]
    value: usize,
}
#[derive(Debug, Clone)]
pub struct SubEvent;

pub struct SubContext {
    #[allow(dead_code)]
    value: usize,
}

#[finny_fsm]
fn build_sub_fsm(mut fsm: FsmBuilder<SubStateMachine, SubContext>) -> BuiltFsm {
    fsm.initial_state::<SubStateA>();
    fsm.state::<SubStateA>()
        .on_entry(|state, _ctx| {
            state.value += 1;
        })
        .on_event::<SubEvent>()
        .transition_to::<SubStateB>()
        .action(|_ev, _ctx, state_a, _state_b| {
            state_a.value += 1;
        });

    fsm.state::<SubStateB>();
    fsm.build()
}

#[derive(Default)]
pub struct SecondSubStateA {
    value: usize,
}
#[derive(Default)]
pub struct SecondSubStateB {
    #[allow(dead_code)]
    value: usize,
}
#[derive(Debug, Clone)]
pub struct SecondSubEvent;

pub struct SecondSubContext {
    #[allow(dead_code)]
    value: usize,
}

#[finny_fsm]
fn build_second_sub_fsm(mut fsm: FsmBuilder<SecondSubStateMachine, SecondSubContext>) -> BuiltFsm {
    fsm.initial_state::<SecondSubStateA>();
    fsm.state::<SecondSubStateA>()
        .on_entry(|state, _ctx| {
            state.value += 1;
        })
        .on_event::<SecondSubEvent>()
        .transition_to::<SecondSubStateB>()
        .action(|_ev, _ctx, state_a, _state_b| {
            state_a.value += 1;
        });

    fsm.state::<SecondSubStateB>();
    fsm.build()
}

#[test]
fn test_sub() -> FsmResult<()> {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = std::sync::Mutex::new(drain).fuse();

    let logger = slog::Logger::root(drain, o!());

    let mut fsm = StateMachine::new_with(
        MainContext::default(),
        FsmEventQueueVec::new(),
        InspectSlog::new(Some(logger)),
        FsmTimersNull,
    )?;

    fsm.start()?;
    assert_eq!(
        FsmCurrentState::State(StateMachineCurrentState::StateA),
        fsm.get_current_states()[0]
    );

    fsm.dispatch(Event)?;

    assert_eq!(
        FsmCurrentState::State(StateMachineCurrentState::SubStateMachine),
        fsm.get_current_states()[0]
    );
    let sub: &SubStateMachine = fsm.get_state();
    assert_eq!(
        FsmCurrentState::State(SubStateMachineCurrentState::SubStateA),
        sub.get_current_states()[0]
    );
    let state: &SubStateA = sub.get_state();
    assert_eq!(1, state.value);

    let ev: SubStateMachineEvents = SubEvent.into();
    fsm.dispatch(ev)?;

    assert_eq!(
        FsmCurrentState::State(StateMachineCurrentState::SubStateMachine),
        fsm.get_current_states()[0]
    );
    let sub: &SubStateMachine = fsm.get_state();
    assert_eq!(
        FsmCurrentState::State(SubStateMachineCurrentState::SubStateB),
        sub.get_current_states()[0]
    );
    let state: &SubStateA = sub.get_state();
    assert_eq!(2, state.value);

    let res = fsm.dispatch(EventSub { n: 0 });
    assert_eq!(Err(FsmError::NoTransition), res);
    assert_eq!(1, fsm.sub_enter);
    assert_eq!(0, fsm.sub_exit);
    assert_eq!(0, fsm.sub_action);

    fsm.dispatch(EventSub { n: 1 })?;
    assert_eq!(2, fsm.sub_enter);
    assert_eq!(1, fsm.sub_exit);
    assert_eq!(1, fsm.sub_action);

    fsm.dispatch(Event)?;
    assert_eq!(
        FsmCurrentState::State(StateMachineCurrentState::StateA),
        fsm.get_current_states()[0]
    );

    fsm.dispatch(Event)?;
    assert_eq!(
        FsmCurrentState::State(StateMachineCurrentState::SubStateMachine),
        fsm.get_current_states()[0]
    );

    fsm.dispatch(EventSubSecond)?;
    assert_eq!(
        FsmCurrentState::State(StateMachineCurrentState::SecondSubStateMachine),
        fsm.get_current_states()[0]
    );

    Ok(())
}
