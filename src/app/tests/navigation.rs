use super::*;

#[test]
fn navigation_explicitly_leaves_disconnected_selection() {
    let mut app = App::new();
    app.connect(4, metadata("first"));
    app.connect(8, metadata("second"));
    app.disconnect(4);

    app.select_next();

    assert_eq!(app.selected_id, Some(8));
}

#[test]
fn device_selector_visibility_is_explicit() {
    let mut app = App::new();

    app.open_device_selector();
    assert!(app.device_selector_visible);

    app.close_device_selector();
    assert!(!app.device_selector_visible);
}

#[test]
fn tab_navigation_wraps_in_both_directions() {
    let mut app = App::new();

    app.select_previous_tab();
    assert_eq!(app.active_tab, AppTab::Timing);
    app.select_next_tab();
    assert_eq!(app.active_tab, AppTab::Dashboard);
}
