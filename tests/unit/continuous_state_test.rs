// T056: Unit tests for continuous execution state management

use wasmflow::graph::node::{ContinuousExecutionState, ContinuousNodeConfig, ContinuousRuntimeState};
use wasmflow::graph::state::{validate_transition, can_start, can_stop};

#[test]
fn test_state_transition_idle_to_starting() {
    let result = validate_transition(
        ContinuousExecutionState::Idle,
        ContinuousExecutionState::Starting
    );
    assert!(result.is_ok(), "Should allow transition from Idle to Starting");
}

#[test]
fn test_state_transition_starting_to_running() {
    let result = validate_transition(
        ContinuousExecutionState::Starting,
        ContinuousExecutionState::Running
    );
    assert!(result.is_ok(), "Should allow transition from Starting to Running");
}

#[test]
fn test_state_transition_running_to_stopping() {
    let result = validate_transition(
        ContinuousExecutionState::Running,
        ContinuousExecutionState::Stopping
    );
    assert!(result.is_ok(), "Should allow transition from Running to Stopping");
}

#[test]
fn test_state_transition_stopping_to_stopped() {
    let result = validate_transition(
        ContinuousExecutionState::Stopping,
        ContinuousExecutionState::Stopped
    );
    assert!(result.is_ok(), "Should allow transition from Stopping to Stopped");
}

#[test]
fn test_state_transition_to_error_from_any() {
    // Error state can be reached from any state
    let states = vec![
        ContinuousExecutionState::Idle,
        ContinuousExecutionState::Starting,
        ContinuousExecutionState::Running,
        ContinuousExecutionState::Stopping,
        ContinuousExecutionState::Stopped,
    ];

    for state in states {
        let result = validate_transition(state, ContinuousExecutionState::Error);
        assert!(result.is_ok(), "Should allow transition from {:?} to Error", state);
    }
}

#[test]
fn test_state_transition_invalid_idle_to_running() {
    let result = validate_transition(
        ContinuousExecutionState::Idle,
        ContinuousExecutionState::Running
    );
    assert!(result.is_err(), "Should not allow transition from Idle directly to Running");
}

#[test]
fn test_state_transition_invalid_running_to_idle() {
    let result = validate_transition(
        ContinuousExecutionState::Running,
        ContinuousExecutionState::Idle
    );
    assert!(result.is_err(), "Should not allow transition from Running directly to Idle");
}

#[test]
fn test_can_start_from_idle() {
    let config = ContinuousNodeConfig {
        supports_continuous: true,
        enabled: true,
        runtime_state: ContinuousRuntimeState {
            execution_state: ContinuousExecutionState::Idle,
            ..Default::default()
        },
    };

    assert!(can_start(&config), "Should be able to start from Idle state");
}

#[test]
fn test_can_start_from_stopped() {
    let config = ContinuousNodeConfig {
        supports_continuous: true,
        enabled: true,
        runtime_state: ContinuousRuntimeState {
            execution_state: ContinuousExecutionState::Stopped,
            ..Default::default()
        },
    };

    assert!(can_start(&config), "Should be able to start from Stopped state");
}

#[test]
fn test_can_start_from_error() {
    let config = ContinuousNodeConfig {
        supports_continuous: true,
        enabled: true,
        runtime_state: ContinuousRuntimeState {
            execution_state: ContinuousExecutionState::Error,
            last_error: Some("Previous error".to_string()),
            ..Default::default()
        },
    };

    assert!(can_start(&config), "Should be able to start from Error state (restart after error)");
}

#[test]
fn test_cannot_start_from_running() {
    let config = ContinuousNodeConfig {
        supports_continuous: true,
        enabled: true,
        runtime_state: ContinuousRuntimeState {
            execution_state: ContinuousExecutionState::Running,
            ..Default::default()
        },
    };

    assert!(!can_start(&config), "Should not be able to start from Running state");
}

#[test]
fn test_cannot_start_when_disabled() {
    let config = ContinuousNodeConfig {
        supports_continuous: true,
        enabled: false, // Disabled
        runtime_state: ContinuousRuntimeState {
            execution_state: ContinuousExecutionState::Idle,
            ..Default::default()
        },
    };

    assert!(!can_start(&config), "Should not be able to start when continuous execution is disabled");
}

#[test]
fn test_can_stop_from_running() {
    let config = ContinuousNodeConfig {
        supports_continuous: true,
        enabled: true,
        runtime_state: ContinuousRuntimeState {
            execution_state: ContinuousExecutionState::Running,
            ..Default::default()
        },
    };

    assert!(can_stop(&config), "Should be able to stop from Running state");
}

#[test]
fn test_can_stop_from_starting() {
    let config = ContinuousNodeConfig {
        supports_continuous: true,
        enabled: true,
        runtime_state: ContinuousRuntimeState {
            execution_state: ContinuousExecutionState::Starting,
            ..Default::default()
        },
    };

    assert!(can_stop(&config), "Should be able to stop from Starting state");
}

#[test]
fn test_cannot_stop_from_idle() {
    let config = ContinuousNodeConfig {
        supports_continuous: true,
        enabled: true,
        runtime_state: ContinuousRuntimeState {
            execution_state: ContinuousExecutionState::Idle,
            ..Default::default()
        },
    };

    assert!(!can_stop(&config), "Should not be able to stop from Idle state");
}

#[test]
fn test_runtime_state_default() {
    let state = ContinuousRuntimeState::default();

    assert_eq!(state.execution_state, ContinuousExecutionState::Idle);
    assert_eq!(state.iterations, 0);
    assert!(state.started_at.is_none());
    assert!(state.last_error.is_none());
}

#[test]
fn test_config_default() {
    let config = ContinuousNodeConfig::default();

    assert!(!config.supports_continuous);
    assert!(!config.enabled);
    assert_eq!(config.runtime_state.execution_state, ContinuousExecutionState::Idle);
}
