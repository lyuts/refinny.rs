extern crate finny;

use finny::{bundled::derive_more, finny_fsm, FsmCurrentState, FsmFactory, FsmResult};

#[derive(Default)]
pub struct StateA;
#[derive(Default)]
pub struct StateB;
#[derive(Default)]
pub struct StateX;
#[derive(Default)]
pub struct StateY;
#[derive(Clone)]
pub struct Event;

#[finny_fsm]
fn build_fsm(mut fsm: FsmBuilder<StateMachine, ()>) -> BuiltFsm {
    fsm.initial_states::<(StateA, StateX)>();

    // region 1

    fsm.state::<StateA>();
    fsm.state::<StateB>();

    fsm.state::<StateA>()
        .on_event::<Event>()
        .transition_to::<StateB>();

    // region 2

    fsm.state::<StateX>();
    fsm.state::<StateY>();

    fsm.state::<StateX>()
        .on_event::<Event>()
        .transition_to::<StateY>();

    fsm.build()
}

#[test]
fn test_regions() -> FsmResult<()> {
    let mut fsm = StateMachine::new(())?;

    fsm.start()?;
    let current_states = fsm.get_current_states();
    assert_eq!(
        [
            FsmCurrentState::State(StateMachineCurrentState::StateA),
            FsmCurrentState::State(StateMachineCurrentState::StateX)
        ],
        current_states
    );

    Ok(())
}
