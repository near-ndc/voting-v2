use near_sdk::{serde::Serialize, serde_json::json};

use common_contracts::events::{EventPayload, NearEvent};

use crate::types::Status;

fn emit_event<T: Serialize>(event: EventPayload<T>) {
    NearEvent {
        standard: "ndc-snapshot",
        version: "1.0.0",
        event,
    }
    .emit();
}

pub fn emit_phase_change(phase: Status) {
    let (attempt, phase_name) = phase.event_info();
    emit_event(EventPayload {
        event: "phase_change",
        data: json!({ "phase": phase_name, "attempt": attempt}),
    });
}

#[cfg(test)]
mod unit_tests {
    use near_sdk::test_utils;

    use super::*;

    #[test]
    fn log_vote() {
        let expected1 = r#"EVENT_JSON:{"standard":"ndc-snapshot","version":"1.0.0","event":"phase_change","data":{"attempt":3,"phase":"RegistrationEnded"}}"#;
        emit_phase_change(Status::RegistrationEnded(3));
        assert_eq!(vec![expected1], test_utils::get_logs());
    }
}
