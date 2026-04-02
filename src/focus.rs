//! Focus management for keyboard navigation.
//!
//! [`FocusManager`] maintains a tab-order list and tracks which widget
//! currently holds keyboard focus.  Tab advances focus forward; Shift+Tab
//! moves it backward.

use crate::widgets::WidgetId;

/// Manages keyboard focus across the widget tree.
pub struct FocusManager {
    /// The widget that currently holds focus, if any.
    focused: Option<WidgetId>,
    /// Tab-order list of focusable widgets.
    focus_order: Vec<WidgetId>,
}

impl FocusManager {
    /// Create a new, empty focus manager with no focused widget.
    pub fn new() -> Self {
        Self {
            focused: None,
            focus_order: Vec::new(),
        }
    }

    /// Return the currently focused widget, if any.
    pub fn focused(&self) -> Option<WidgetId> {
        self.focused
    }

    /// Set focus to a specific widget.
    ///
    /// The widget does **not** need to be in `focus_order` — this allows
    /// programmatic focus on any widget.  However, Tab / Shift+Tab navigation
    /// will only cycle through registered widgets.
    pub fn set_focus(&mut self, id: WidgetId) {
        self.focused = Some(id);
    }

    /// Clear focus so that no widget is focused.
    pub fn clear_focus(&mut self) {
        self.focused = None;
    }

    /// Move focus to the next widget in tab order.
    ///
    /// If nothing is focused, focuses the first widget.  If the currently
    /// focused widget is not in the tab order, focuses the first widget.
    /// Wraps around at the end.
    pub fn focus_next(&mut self) {
        if self.focus_order.is_empty() {
            return;
        }

        let next_idx = match self.focused {
            Some(current) => match self.focus_order.iter().position(|&id| id == current) {
                Some(idx) => (idx + 1) % self.focus_order.len(),
                None => 0,
            },
            None => 0,
        };

        self.focused = Some(self.focus_order[next_idx]);
    }

    /// Move focus to the previous widget in tab order.
    ///
    /// If nothing is focused, focuses the last widget.  Wraps around at the
    /// beginning.
    pub fn focus_prev(&mut self) {
        if self.focus_order.is_empty() {
            return;
        }

        let prev_idx = match self.focused {
            Some(current) => match self.focus_order.iter().position(|&id| id == current) {
                Some(0) => self.focus_order.len() - 1,
                Some(idx) => idx - 1,
                None => self.focus_order.len() - 1,
            },
            None => self.focus_order.len() - 1,
        };

        self.focused = Some(self.focus_order[prev_idx]);
    }

    /// Register a widget in the tab order.
    ///
    /// The widget is appended at the end.  Duplicate registrations are
    /// silently ignored.
    pub fn register(&mut self, id: WidgetId) {
        if !self.focus_order.contains(&id) {
            self.focus_order.push(id);
        }
    }

    /// Remove a widget from the tab order.
    ///
    /// If the removed widget was focused, focus is cleared.
    pub fn unregister(&mut self, id: WidgetId) {
        self.focus_order.retain(|&x| x != id);
        if self.focused == Some(id) {
            self.focused = None;
        }
    }

    /// Check whether a specific widget is currently focused.
    pub fn is_focused(&self, id: WidgetId) -> bool {
        self.focused == Some(id)
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> WidgetId {
        WidgetId::manual(n)
    }

    #[test]
    fn new_focus_manager_has_no_focus() {
        let fm = FocusManager::new();
        assert_eq!(fm.focused(), None);
    }

    #[test]
    fn set_and_get_focus() {
        let mut fm = FocusManager::new();
        fm.set_focus(id(1));
        assert_eq!(fm.focused(), Some(id(1)));
    }

    #[test]
    fn clear_focus() {
        let mut fm = FocusManager::new();
        fm.set_focus(id(1));
        fm.clear_focus();
        assert_eq!(fm.focused(), None);
    }

    #[test]
    fn is_focused() {
        let mut fm = FocusManager::new();
        fm.set_focus(id(1));
        assert!(fm.is_focused(id(1)));
        assert!(!fm.is_focused(id(2)));
    }

    #[test]
    fn register_prevents_duplicates() {
        let mut fm = FocusManager::new();
        fm.register(id(1));
        fm.register(id(1));
        assert_eq!(fm.focus_order.len(), 1);
    }

    #[test]
    fn unregister_clears_focus_if_removed() {
        let mut fm = FocusManager::new();
        fm.register(id(1));
        fm.set_focus(id(1));
        fm.unregister(id(1));
        assert_eq!(fm.focused(), None);
        assert!(fm.focus_order.is_empty());
    }

    #[test]
    fn unregister_does_not_clear_focus_of_other() {
        let mut fm = FocusManager::new();
        fm.register(id(1));
        fm.register(id(2));
        fm.set_focus(id(1));
        fm.unregister(id(2));
        assert_eq!(fm.focused(), Some(id(1)));
    }

    #[test]
    fn focus_next_empty_order_is_noop() {
        let mut fm = FocusManager::new();
        fm.focus_next();
        assert_eq!(fm.focused(), None);
    }

    #[test]
    fn focus_prev_empty_order_is_noop() {
        let mut fm = FocusManager::new();
        fm.focus_prev();
        assert_eq!(fm.focused(), None);
    }

    #[test]
    fn focus_next_from_none_goes_to_first() {
        let mut fm = FocusManager::new();
        fm.register(id(1));
        fm.register(id(2));
        fm.register(id(3));
        fm.focus_next();
        assert_eq!(fm.focused(), Some(id(1)));
    }

    #[test]
    fn focus_prev_from_none_goes_to_last() {
        let mut fm = FocusManager::new();
        fm.register(id(1));
        fm.register(id(2));
        fm.register(id(3));
        fm.focus_prev();
        assert_eq!(fm.focused(), Some(id(3)));
    }

    #[test]
    fn focus_next_cycles_forward() {
        let mut fm = FocusManager::new();
        fm.register(id(1));
        fm.register(id(2));
        fm.register(id(3));
        fm.set_focus(id(1));

        fm.focus_next();
        assert_eq!(fm.focused(), Some(id(2)));

        fm.focus_next();
        assert_eq!(fm.focused(), Some(id(3)));

        // Wraps around
        fm.focus_next();
        assert_eq!(fm.focused(), Some(id(1)));
    }

    #[test]
    fn focus_prev_cycles_backward() {
        let mut fm = FocusManager::new();
        fm.register(id(1));
        fm.register(id(2));
        fm.register(id(3));
        fm.set_focus(id(3));

        fm.focus_prev();
        assert_eq!(fm.focused(), Some(id(2)));

        fm.focus_prev();
        assert_eq!(fm.focused(), Some(id(1)));

        // Wraps around
        fm.focus_prev();
        assert_eq!(fm.focused(), Some(id(3)));
    }

    #[test]
    fn focus_next_from_unregistered_goes_to_first() {
        let mut fm = FocusManager::new();
        fm.register(id(1));
        fm.register(id(2));
        fm.set_focus(id(99)); // not in order
        fm.focus_next();
        assert_eq!(fm.focused(), Some(id(1)));
    }

    #[test]
    fn focus_prev_from_unregistered_goes_to_last() {
        let mut fm = FocusManager::new();
        fm.register(id(1));
        fm.register(id(2));
        fm.set_focus(id(99)); // not in order
        fm.focus_prev();
        assert_eq!(fm.focused(), Some(id(2)));
    }

    #[test]
    fn single_widget_focus_next_stays() {
        let mut fm = FocusManager::new();
        fm.register(id(1));
        fm.set_focus(id(1));
        fm.focus_next();
        assert_eq!(fm.focused(), Some(id(1)));
    }

    #[test]
    fn single_widget_focus_prev_stays() {
        let mut fm = FocusManager::new();
        fm.register(id(1));
        fm.set_focus(id(1));
        fm.focus_prev();
        assert_eq!(fm.focused(), Some(id(1)));
    }

    #[test]
    fn default_is_same_as_new() {
        let fm = FocusManager::default();
        assert_eq!(fm.focused(), None);
        assert!(fm.focus_order.is_empty());
    }
}
