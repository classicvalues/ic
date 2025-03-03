use ic_base_types::{NumBytes, NumSeconds};
use ic_interfaces::execution_environment::{CanisterOutOfCyclesError, SystemApi};
use ic_registry_subnet_type::SubnetType;
use ic_replicated_state::{StateError, SystemState};
use ic_system_api::{SystemStateAccessor, SystemStateAccessorDirect};
use ic_test_utilities::{
    cycles_account_manager::CyclesAccountManagerBuilder,
    state::SystemStateBuilder,
    types::{
        ids::{canister_test_id, user_test_id},
        messages::{RequestBuilder, ResponseBuilder},
    },
};
use ic_types::{
    messages::MAX_INTER_CANISTER_PAYLOAD_IN_BYTES, ComputeAllocation, Cycles, NumInstructions,
};
use std::{convert::From, sync::Arc};

mod common;
use common::*;

const MAX_NUM_INSTRUCTIONS: NumInstructions = NumInstructions::new(1 << 30);
const INITIAL_CYCLES: Cycles = Cycles::new(5_000_000_000_000);

#[test]
fn push_output_request_fails_not_enough_cycles_for_request() {
    let request = RequestBuilder::default()
        .sender(canister_test_id(0))
        .build();

    let cycles_account_manager = Arc::new(
        CyclesAccountManagerBuilder::new()
            .with_max_num_instructions(MAX_NUM_INSTRUCTIONS)
            .build(),
    );

    let xnet_cost = cycles_account_manager.xnet_call_performed_fee();
    let request_payload_cost =
        cycles_account_manager.xnet_call_bytes_transmitted_fee(request.payload_size_bytes());
    let response_reservation =
        cycles_account_manager.xnet_call_bytes_transmitted_fee(MAX_INTER_CANISTER_PAYLOAD_IN_BYTES);
    let total_cost = xnet_cost
        + request_payload_cost
        + response_reservation
        + cycles_account_manager.execution_cost(MAX_NUM_INSTRUCTIONS);

    // Set cycles balance low enough that not even the cost for transferring
    // the request is covered.
    let system_state = SystemState::new_running(
        canister_test_id(0),
        user_test_id(1).get(),
        request_payload_cost - Cycles::from(10),
        NumSeconds::from(100_000),
    );

    let system_state_accessor =
        SystemStateAccessorDirect::new(system_state, cycles_account_manager);

    assert_eq!(
        system_state_accessor.push_output_request(
            NumBytes::from(0),
            ComputeAllocation::default(),
            request.clone()
        ),
        Err((
            StateError::CanisterOutOfCycles(CanisterOutOfCyclesError {
                canister_id: canister_test_id(0),
                available: request_payload_cost - Cycles::from(10),
                requested: total_cost,
                threshold: Cycles::from(0),
            }),
            request
        ))
    );
}

#[test]
fn push_output_request_fails_not_enough_cycles_for_response() {
    let request = RequestBuilder::default()
        .sender(canister_test_id(0))
        .build();

    let cycles_account_manager = Arc::new(
        CyclesAccountManagerBuilder::new()
            .with_max_num_instructions(MAX_NUM_INSTRUCTIONS)
            .build(),
    );

    let xnet_cost = cycles_account_manager.xnet_call_performed_fee();
    let request_payload_cost =
        cycles_account_manager.xnet_call_bytes_transmitted_fee(request.payload_size_bytes());
    let response_reservation =
        cycles_account_manager.xnet_call_bytes_transmitted_fee(MAX_INTER_CANISTER_PAYLOAD_IN_BYTES);
    let total_cost = xnet_cost
        + request_payload_cost
        + response_reservation
        + cycles_account_manager.execution_cost(MAX_NUM_INSTRUCTIONS);

    // Set cycles balance to a number that is enough to cover for the request
    // transfer but not to cover the cost of processing the expected response.
    let system_state = SystemState::new_running(
        canister_test_id(0),
        user_test_id(1).get(),
        total_cost - Cycles::from(10),
        NumSeconds::from(100_000),
    );

    let system_state_accessor =
        SystemStateAccessorDirect::new(system_state, cycles_account_manager);

    assert_eq!(
        system_state_accessor.push_output_request(
            NumBytes::from(0),
            ComputeAllocation::default(),
            request.clone()
        ),
        Err((
            StateError::CanisterOutOfCycles(CanisterOutOfCyclesError {
                canister_id: canister_test_id(0),
                available: total_cost - Cycles::from(10),
                requested: total_cost,
                threshold: Cycles::from(0),
            }),
            request
        ))
    );
}

#[test]
fn push_output_request_succeeds_with_enough_cycles() {
    let cycles_account_manager = Arc::new(
        CyclesAccountManagerBuilder::new()
            .with_max_num_instructions(MAX_NUM_INSTRUCTIONS)
            .build(),
    );

    let system_state = SystemState::new_running(
        canister_test_id(0),
        user_test_id(1).get(),
        INITIAL_CYCLES,
        NumSeconds::from(100_000),
    );

    let system_state_accessor =
        SystemStateAccessorDirect::new(system_state, Arc::clone(&cycles_account_manager));

    assert_eq!(
        system_state_accessor.push_output_request(
            NumBytes::from(0),
            ComputeAllocation::default(),
            RequestBuilder::default()
                .sender(canister_test_id(0))
                .build(),
        ),
        Ok(())
    );
}

#[test]
fn correct_charging_source_canister_for_a_request() {
    let subnet_type = SubnetType::Application;
    let cycles_account_manager = Arc::new(
        CyclesAccountManagerBuilder::new()
            .with_max_num_instructions(MAX_NUM_INSTRUCTIONS)
            .with_subnet_type(subnet_type)
            .build(),
    );
    let system_state = SystemState::new_running(
        canister_test_id(0),
        user_test_id(1).get(),
        INITIAL_CYCLES,
        NumSeconds::from(100_000),
    );

    let initial_cycles_balance = system_state.cycles_balance;

    let system_state_accessor =
        SystemStateAccessorDirect::new(system_state, Arc::clone(&cycles_account_manager));

    let request = RequestBuilder::default()
        .sender(canister_test_id(0))
        .receiver(canister_test_id(1))
        .build();

    let xnet_cost = cycles_account_manager.xnet_call_performed_fee();
    let request_payload_cost =
        cycles_account_manager.xnet_call_bytes_transmitted_fee(request.payload_size_bytes());
    // Which should result in refunding everything except the response payload cost
    let response_reservation =
        cycles_account_manager.xnet_call_bytes_transmitted_fee(MAX_INTER_CANISTER_PAYLOAD_IN_BYTES);
    let total_cost = xnet_cost
        + request_payload_cost
        + response_reservation
        + cycles_account_manager.execution_cost(MAX_NUM_INSTRUCTIONS);

    // Enqueue the Request.
    system_state_accessor
        .push_output_request(NumBytes::from(0), ComputeAllocation::default(), request)
        .unwrap();

    // Assume the destination canister got the message and prepared a response
    let mut response = ResponseBuilder::default()
        .respondent(canister_test_id(1))
        .originator(canister_test_id(0))
        .build();

    // The response will find its way into the
    // ExecutionEnvironmentImpl::execute_canister_response()
    // => Mock the response_cycles_refund() invocation from the
    // execute_canister_response()
    let mut system_state = system_state_accessor.release_system_state();
    cycles_account_manager.response_cycles_refund(&mut system_state, &mut response);

    // MAX_NUM_INSTRUCTIONS also gets partially refunded in the real
    // ExecutionEnvironmentImpl::execute_canister_response()
    assert_eq!(
        initial_cycles_balance - total_cost
            + cycles_account_manager.xnet_call_bytes_transmitted_fee(
                MAX_INTER_CANISTER_PAYLOAD_IN_BYTES - response.response_payload.size_of()
            ),
        system_state.cycles_balance
    );
}

#[test]
fn mint_all_cycles() {
    let cycles_account_manager = CyclesAccountManagerBuilder::new()
        .with_subnet_type(SubnetType::System)
        .build();
    let system_state = SystemStateBuilder::new().build();
    let mut api = get_system_api(get_update_api_type(), system_state, cycles_account_manager);
    let balance_before = api.ic0_canister_cycle_balance().unwrap();

    let amount = 50;
    assert_eq!(api.ic0_mint_cycles(amount), Ok(amount));
    assert_eq!(
        api.ic0_canister_cycle_balance().unwrap() - balance_before,
        amount
    );
}

#[test]
fn mint_cycles_above_max() {
    let cycles_account_manager = CyclesAccountManagerBuilder::new()
        .with_subnet_type(SubnetType::System)
        .build();
    let mut system_state = SystemStateBuilder::new().build();

    // Set cycles balance to max - 10.
    cycles_account_manager.add_cycles(&mut system_state, CYCLES_LIMIT_PER_CANISTER);
    cycles_account_manager
        .withdraw_cycles_for_transfer(
            &mut system_state,
            NumBytes::from(0),
            ComputeAllocation::default(),
            Cycles::from(10),
        )
        .unwrap();

    let mut api = get_system_api(get_update_api_type(), system_state, cycles_account_manager);
    let balance_before = api.ic0_canister_cycle_balance().unwrap();

    let amount = 50;
    // Canisters on the System subnet can hold any amount of cycles
    assert_eq!(api.ic0_mint_cycles(amount), Ok(amount));
    assert_eq!(
        api.ic0_canister_cycle_balance().unwrap() - balance_before,
        amount
    );
}

#[test]
fn mint_cycles_fails_caller_not_on_nns() {
    let system_state = SystemStateBuilder::default().build();
    let cycles_account_manager = CyclesAccountManagerBuilder::new().build();
    let mut api = get_system_api(get_update_api_type(), system_state, cycles_account_manager);

    let balance_before = api.ic0_canister_cycle_balance().unwrap();

    assert!(api.ic0_mint_cycles(50).is_err());
    assert_eq!(
        api.ic0_canister_cycle_balance().unwrap() - balance_before,
        0
    );
}
