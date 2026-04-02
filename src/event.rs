use crate::canvas::Rect;

#[derive(Debug, Clone, Copy)]
pub enum UiEvent {
    MouseMove { x: f32, y: f32 },
    MouseDown { x: f32, y: f32 },
    MouseUp { x: f32, y: f32 },
}

#[derive(Debug, Default)]
pub struct EventState {
    pub hovered: Option<u64>,
    pub pressed: Option<u64>,
    pub cursor: (f32, f32),
    dirty: Option<Rect>,
    needs_redraw: bool,
}

impl EventState {
    /// Mark a region as needing repaint. Regions are merged into a single
    /// bounding box so the bookkeeping stays O(1).
    pub fn mark_dirty(&mut self, rect: Rect) {
        self.dirty = Some(match self.dirty {
            Some(existing) => existing.union(&rect),
            None => rect,
        });
        self.needs_redraw = true;
    }

    /// Mark the entire framebuffer as dirty (e.g. after resize).
    pub fn mark_full_redraw(&mut self) {
        self.needs_redraw = true;
    }

    /// Consume the dirty flag. Returns true if a redraw is needed.
    pub fn take_needs_redraw(&mut self) -> bool {
        let v = self.needs_redraw;
        self.needs_redraw = false;
        self.dirty = None;
        v
    }

    /// Peek at the current dirty bounding box (if any).
    pub fn dirty_rect(&self) -> Option<Rect> {
        self.dirty
    }
}
