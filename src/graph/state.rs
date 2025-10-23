//! State transition validation for continuous execution nodes

use super::node::ContinuousExecutionState;

/// Validate if a state transition is allowed
pub fn validate_transition(from: ContinuousExecutionState, to: ContinuousExecutionState) -> bool {
    use ContinuousExecutionState::*;

    match (from, to) {
        // From Idle
        (Idle, Starting) => true,
        (Idle, Idle) => true,

        // From Starting
        (Starting, Running) => true,
        (Starting, Error) => true,

        // From Running
        (Running, Stopping) => true,
        (Running, Error) => true,
        (Running, Running) => true,

        // From Stopping
        (Stopping, Stopped) => true,
        (Stopping, Idle) => true,
        (Stopping, Error) => true,

        // From Stopped
        (Stopped, Idle) => true,
        (Stopped, Starting) => true,

        // From Error
        (Error, Idle) => true,
        (Error, Starting) => true,

        // All other transitions are invalid
        _ => false,
    }
}

/// Check if a node can be started from its current state
pub fn can_start(state: ContinuousExecutionState) -> bool {
    matches!(
        state,
        ContinuousExecutionState::Idle | ContinuousExecutionState::Stopped | ContinuousExecutionState::Error
    )
}

/// Check if a node can be stopped from its current state
pub fn can_stop(state: ContinuousExecutionState) -> bool {
    matches!(
        state,
        ContinuousExecutionState::Running | ContinuousExecutionState::Starting
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        use ContinuousExecutionState::*;

        // Valid startup sequence
        assert!(validate_transition(Idle, Starting));
        assert!(validate_transition(Starting, Running));

        // Valid shutdown sequence
        assert!(validate_transition(Running, Stopping));
        assert!(validate_transition(Stopping, Stopped));
        assert!(validate_transition(Stopped, Idle));

        // Error transitions
        assert!(validate_transition(Running, Error));
        assert!(validate_transition(Error, Idle));
    }

    #[test]
    fn test_invalid_transitions() {
        use ContinuousExecutionState::*;

        // Can't go directly from Idle to Running
        assert!(!validate_transition(Idle, Running));

        // Can't go from Starting to Stopped
        assert!(!validate_transition(Starting, Stopped));

        // Can't go from Stopped to Running
        assert!(!validate_transition(Stopped, Running));
    }

    #[test]
    fn test_can_start() {
        use ContinuousExecutionState::*;

        assert!(can_start(Idle));
        assert!(can_start(Stopped));
        assert!(can_start(Error));

        assert!(!can_start(Starting));
        assert!(!can_start(Running));
        assert!(!can_start(Stopping));
    }

    #[test]
    fn test_can_stop() {
        use ContinuousExecutionState::*;

        assert!(can_stop(Running));
        assert!(can_stop(Starting));

        assert!(!can_stop(Idle));
        assert!(!can_stop(Stopped));
        assert!(!can_stop(Stopping));
        assert!(!can_stop(Error));
    }
}
