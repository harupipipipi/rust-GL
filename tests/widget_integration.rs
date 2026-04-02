//! Integration tests for widget event handling and state transitions.

#[macro_use]
#[path = "test_utils.rs"]
mod test_utils;

use rust2d_ui::*;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

// ─────────────────────────────────────────────────────────────
// Button hover → pressed → release full cycle
// ─────────────────────────────────────────────────────────────

#[test]
fn button_full_state_transition() {
    let click_count = Arc::new(AtomicU32::new(0));
    let cc = click_count.clone();

    let mut btn = Button::new(WidgetId::manual(1), "Test")
        .on_click(move || {
            cc.fetch_add(1, Ordering::SeqCst);
        });
    let layout = LayoutNode::new(WidgetId::manual(1), 0, 0, 100, 40);
    let mut es = EventState::default();

    // 1. Move outside — no change (already not hovered).
    let changed = btn.handle_event(
        &UiEvent::MouseMove { x: 200.0, y: 200.0 },
        &mut es,
        &layout,
    );
    assert!(!changed, "move outside initially should not change");

    // 2. Hover in.
    let changed = btn.handle_event(
        &UiEvent::MouseMove { x: 50.0, y: 20.0 },
        &mut es,
        &layout,
    );
    assert!(changed, "entering hover region should change");

    // 3. Mouse down.
    let changed = btn.handle_event(
        &UiEvent::MouseDown { x: 50.0, y: 20.0 },
        &mut es,
        &layout,
    );
    assert!(changed, "mouse down should change");
    assert_eq!(es.pressed, Some(WidgetId::manual(1)));
    assert_eq!(click_count.load(Ordering::SeqCst), 0, "click not yet fired");

    // 4. Mouse up inside — fires click.
    let changed = btn.handle_event(
        &UiEvent::MouseUp { x: 50.0, y: 20.0 },
        &mut es,
        &layout,
    );
    assert!(changed, "mouse up should change");
    assert_eq!(click_count.load(Ordering::SeqCst), 1, "click should fire");

    // 5. Leave hover.
    let changed = btn.handle_event(
        &UiEvent::MouseMove { x: 200.0, y: 200.0 },
        &mut es,
        &layout,
    );
    assert!(changed, "leaving hover should change");
}

#[test]
fn button_press_then_release_outside_no_click() {
    let click_count = Arc::new(AtomicU32::new(0));
    let cc = click_count.clone();

    let mut btn = Button::new(WidgetId::manual(2), "Test")
        .on_click(move || {
            cc.fetch_add(1, Ordering::SeqCst);
        });
    let layout = LayoutNode::new(WidgetId::manual(2), 0, 0, 100, 40);
    let mut es = EventState::default();

    btn.handle_event(&UiEvent::MouseDown { x: 50.0, y: 20.0 }, &mut es, &layout);
    btn.handle_event(&UiEvent::MouseUp { x: 200.0, y: 200.0 }, &mut es, &layout);

    assert_eq!(
        click_count.load(Ordering::SeqCst),
        0,
        "should NOT fire click"
    );
}

#[test]
fn button_double_hover_no_duplicate_change() {
    let mut btn = Button::new(WidgetId::manual(3), "Test");
    let layout = LayoutNode::new(WidgetId::manual(3), 0, 0, 100, 40);
    let mut es = EventState::default();

    let c1 = btn.handle_event(
        &UiEvent::MouseMove { x: 50.0, y: 20.0 },
        &mut es,
        &layout,
    );
    assert!(c1);

    let c2 = btn.handle_event(
        &UiEvent::MouseMove { x: 51.0, y: 21.0 },
        &mut es,
        &layout,
    );
    assert!(!c2, "already hovered, no state transition");
}

// ─────────────────────────────────────────────────────────────
// Container + 3 Buttons event propagation
// ─────────────────────────────────────────────────────────────

#[test]
fn container_three_buttons_event_propagation() {
    let mut container = Container::new(WidgetId::manual(10));
    container.push(Button::new(WidgetId::manual(11), "A"));
    container.push(Button::new(WidgetId::manual(12), "B"));
    container.push(Button::new(WidgetId::manual(13), "C"));

    let mut root_layout = LayoutNode::new(WidgetId::manual(10), 0, 0, 200, 300);
    root_layout.add_child(LayoutNode::new(WidgetId::manual(11), 0, 0, 200, 40));
    root_layout.add_child(LayoutNode::new(WidgetId::manual(12), 0, 50, 200, 40));
    root_layout.add_child(LayoutNode::new(WidgetId::manual(13), 0, 100, 200, 40));

    let mut es = EventState::default();

    // Hover over button A (y=20).
    let changed = container.handle_event(
        &UiEvent::MouseMove { x: 100.0, y: 20.0 },
        &mut es,
        &root_layout,
    );
    assert!(changed, "hovering over button A should propagate");

    // Hover over button B (y=70).
    let changed = container.handle_event(
        &UiEvent::MouseMove { x: 100.0, y: 70.0 },
        &mut es,
        &root_layout,
    );
    assert!(changed, "hovering over button B should propagate");

    // Mouse down on button C (y=120).
    let changed = container.handle_event(
        &UiEvent::MouseDown { x: 100.0, y: 120.0 },
        &mut es,
        &root_layout,
    );
    assert!(changed, "clicking button C should propagate");
    assert_eq!(es.pressed, Some(WidgetId::manual(13)));
}

// ─────────────────────────────────────────────────────────────
// EventState::take_needs_redraw timing
// ─────────────────────────────────────────────────────────────

#[test]
fn take_needs_redraw_timing() {
    let mut es = EventState::default();

    assert!(!es.take_needs_redraw());

    es.request_redraw();
    assert!(es.take_needs_redraw());
    assert!(!es.take_needs_redraw());

    let mut btn = Button::new(WidgetId::manual(20), "Click");
    let layout = LayoutNode::new(WidgetId::manual(20), 0, 0, 100, 40);

    btn.handle_event(
        &UiEvent::MouseMove { x: 50.0, y: 20.0 },
        &mut es,
        &layout,
    );
    assert!(es.take_needs_redraw(), "hover should request redraw");
    assert!(!es.take_needs_redraw(), "second take should be false");
}

#[test]
fn take_needs_redraw_multiple_events_single_take() {
    let mut es = EventState::default();

    let mut btn = Button::new(WidgetId::manual(21), "X");
    let layout = LayoutNode::new(WidgetId::manual(21), 0, 0, 100, 40);

    btn.handle_event(
        &UiEvent::MouseMove { x: 50.0, y: 20.0 },
        &mut es,
        &layout,
    );
    btn.handle_event(
        &UiEvent::MouseDown { x: 50.0, y: 20.0 },
        &mut es,
        &layout,
    );

    assert!(es.take_needs_redraw());
    assert!(!es.take_needs_redraw());
}

// ─────────────────────────────────────────────────────────────
// WidgetId::manual — same ID on two widgets
// ─────────────────────────────────────────────────────────────

#[test]
fn duplicate_manual_id_both_receive_events() {
    let click_a = Arc::new(AtomicU32::new(0));
    let click_b = Arc::new(AtomicU32::new(0));
    let ca = click_a.clone();
    let cb = click_b.clone();

    let mut container = Container::new(WidgetId::manual(30));
    container.push(
        Button::new(WidgetId::manual(99), "A").on_click(move || {
            ca.fetch_add(1, Ordering::SeqCst);
        }),
    );
    container.push(
        Button::new(WidgetId::manual(99), "B").on_click(move || {
            cb.fetch_add(1, Ordering::SeqCst);
        }),
    );

    let mut root_layout = LayoutNode::new(WidgetId::manual(30), 0, 0, 200, 200);
    root_layout.add_child(LayoutNode::new(WidgetId::manual(99), 0, 0, 200, 40));
    root_layout.add_child(LayoutNode::new(WidgetId::manual(99), 0, 50, 200, 40));

    let mut es = EventState::default();

    // Click in button A's region (y=20).
    container.handle_event(
        &UiEvent::MouseDown { x: 100.0, y: 20.0 },
        &mut es,
        &root_layout,
    );
    container.handle_event(
        &UiEvent::MouseUp { x: 100.0, y: 20.0 },
        &mut es,
        &root_layout,
    );

    assert_eq!(click_a.load(Ordering::SeqCst), 1, "button A should fire");
    assert_eq!(
        click_b.load(Ordering::SeqCst),
        0,
        "button B should NOT fire"
    );
}

#[test]
fn duplicate_manual_id_find_by_id_returns_first() {
    let mut root = LayoutNode::new(WidgetId::manual(30), 0, 0, 200, 200);
    root.add_child(LayoutNode::new(WidgetId::manual(99), 0, 0, 200, 40));
    root.add_child(LayoutNode::new(WidgetId::manual(99), 0, 50, 200, 40));

    let found = root.find_by_id(WidgetId::manual(99)).unwrap();
    assert_eq!(found.bounds.y, 0, "should find first child with that ID");
}
