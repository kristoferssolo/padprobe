use super::*;

#[test]
fn repeated_activation_updates_one_item() {
    let mut checklist = ControlChecklist::default();
    checklist.start(1, []);

    checklist.observe(1, "button:South");
    checklist.observe(1, "button:South");

    let south = &checklist.items()[0];
    assert_eq!(south.status, ChecklistStatus::Observed);
    assert_eq!(south.activation_count, 2);
    assert_eq!(
        checklist
            .items()
            .iter()
            .filter(|item| item.key == "button:South")
            .count(),
        1
    );
}

#[test]
fn unexpected_control_is_retained() {
    let mut checklist = ControlChecklist::default();
    checklist.start(1, []);

    checklist.observe(1, "button:Unknown");

    assert!(
        checklist
            .items()
            .iter()
            .any(|item| item.key == "button:Unknown" && item.unexpected)
    );
}
