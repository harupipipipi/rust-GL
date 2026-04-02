//! Integration tests for layout: Container + children positioning.

#[macro_use]
#[path = "test_utils.rs"]
mod test_utils;

use rust2d_ui::*;

// ─────────────────────────────────────────────────────────────
// Vertical layout: children y increases, gap is reflected
// ─────────────────────────────────────────────────────────────

#[test]
fn vertical_container_children_y_increases() {
    let fm = require_font_manager!();

    let mut root = Container::new(WidgetId::manual(1));
    root.style.direction = LayoutDirection::Vertical;
    root.style.gap = 10.0;
    root.push(Text::new(WidgetId::manual(10), "Line A"));
    root.push(Text::new(WidgetId::manual(11), "Line B"));
    root.push(Text::new(WidgetId::manual(12), "Line C"));

    let constraints = BoxConstraints::loose(400.0, 800.0);
    let layout = root.layout(constraints, 0, 0, &fm);

    assert_eq!(layout.children.len(), 3);

    let y0 = layout.children[0].bounds.y;
    let y1 = layout.children[1].bounds.y;
    let y2 = layout.children[2].bounds.y;

    assert!(y1 > y0, "child1.y ({}) must be > child0.y ({})", y1, y0);
    assert!(y2 > y1, "child2.y ({}) must be > child1.y ({})", y2, y1);
}

#[test]
fn vertical_container_gap_reflected() {
    let fm = require_font_manager!();

    let mut root = Container::new(WidgetId::manual(2));
    root.style.direction = LayoutDirection::Vertical;
    root.style.gap = 20.0;
    root.push(Text::new(WidgetId::manual(20), "A"));
    root.push(Text::new(WidgetId::manual(21), "B"));

    let constraints = BoxConstraints::loose(400.0, 800.0);
    let layout = root.layout(constraints, 0, 0, &fm);

    let child0 = &layout.children[0];
    let child1 = &layout.children[1];

    let gap_actual = child1.bounds.y - (child0.bounds.y + child0.bounds.height as i32);
    assert!(
        (gap_actual - 20).abs() <= 1,
        "gap between children = {}, expected ~20",
        gap_actual
    );
}

// ─────────────────────────────────────────────────────────────
// Horizontal layout
// ─────────────────────────────────────────────────────────────

#[test]
fn horizontal_container_children_x_increases() {
    let fm = require_font_manager!();

    let mut root = Container::new(WidgetId::manual(3));
    root.style.direction = LayoutDirection::Horizontal;
    root.style.gap = 10.0;
    root.push(Text::new(WidgetId::manual(30), "AAA"));
    root.push(Text::new(WidgetId::manual(31), "BBB"));
    root.push(Text::new(WidgetId::manual(32), "CCC"));

    let constraints = BoxConstraints::loose(800.0, 400.0);
    let layout = root.layout(constraints, 0, 0, &fm);

    assert_eq!(layout.children.len(), 3);

    let x0 = layout.children[0].bounds.x;
    let x1 = layout.children[1].bounds.x;
    let x2 = layout.children[2].bounds.x;

    assert!(x1 > x0, "child1.x ({}) must be > child0.x ({})", x1, x0);
    assert!(x2 > x1, "child2.x ({}) must be > child1.x ({})", x2, x1);
}

// ─────────────────────────────────────────────────────────────
// Nested containers
// ─────────────────────────────────────────────────────────────

#[test]
fn nested_container_layout() {
    let fm = require_font_manager!();

    let mut inner = Container::new(WidgetId::manual(41));
    inner.style.direction = LayoutDirection::Vertical;
    inner.push(Text::new(WidgetId::manual(410), "Inner A"));
    inner.push(Text::new(WidgetId::manual(411), "Inner B"));

    let mut outer = Container::new(WidgetId::manual(4));
    outer.style.direction = LayoutDirection::Vertical;
    outer.push(Text::new(WidgetId::manual(40), "Outer Top"));
    outer.push(inner);

    let constraints = BoxConstraints::loose(400.0, 800.0);
    let layout = outer.layout(constraints, 0, 0, &fm);

    assert_eq!(layout.children.len(), 2);
    assert_eq!(layout.children[1].children.len(), 2);

    let outer_text_y = layout.children[0].bounds.y;
    let inner_container_y = layout.children[1].bounds.y;
    assert!(inner_container_y > outer_text_y);

    let inner0_y = layout.children[1].children[0].bounds.y;
    let inner1_y = layout.children[1].children[1].bounds.y;
    assert!(inner1_y > inner0_y);
}

// ─────────────────────────────────────────────────────────────
// Zero-children container
// ─────────────────────────────────────────────────────────────

#[test]
fn empty_container_layout() {
    let fm = require_font_manager!();

    let mut root = Container::new(WidgetId::manual(5));
    root.style.direction = LayoutDirection::Vertical;

    let constraints = BoxConstraints::loose(400.0, 800.0);
    let layout = root.layout(constraints, 0, 0, &fm);

    assert_eq!(layout.children.len(), 0);
    assert_eq!(layout.bounds.height, 0);
}

// ─────────────────────────────────────────────────────────────
// BoxConstraints::tight vs loose
// ─────────────────────────────────────────────────────────────

#[test]
fn box_constraints_tight_vs_loose_different_result() {
    let fm = require_font_manager!();

    let mut root_tight = Container::new(WidgetId::manual(60));
    root_tight.push(Text::new(WidgetId::manual(600), "Short"));

    let mut root_loose = Container::new(WidgetId::manual(61));
    root_loose.push(Text::new(WidgetId::manual(610), "Short"));

    let tight = BoxConstraints::tight(400.0, 300.0);
    let loose = BoxConstraints::loose(400.0, 300.0);

    let layout_tight = root_tight.layout(tight, 0, 0, &fm);
    let layout_loose = root_loose.layout(loose, 0, 0, &fm);

    let h_tight = layout_tight.bounds.height;
    let h_loose = layout_loose.bounds.height;

    assert!(
        h_tight >= h_loose,
        "tight height ({}) should be >= loose height ({})",
        h_tight,
        h_loose
    );
    assert_eq!(h_tight, 300, "tight constraint should force height to 300");
}

#[test]
fn box_constraints_tight_forces_exact_width() {
    let fm = require_font_manager!();

    let mut root = Container::new(WidgetId::manual(70));
    root.push(Text::new(WidgetId::manual(700), "Hello"));

    let tight = BoxConstraints::tight(250.0, 100.0);
    let layout = root.layout(tight, 0, 0, &fm);

    assert_eq!(layout.bounds.width, 250);
}
