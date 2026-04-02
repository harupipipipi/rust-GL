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
    pub dirty_regions: Vec<Rect>,
}

impl EventState {
    pub fn mark_dirty(&mut self, rect: Rect) {
        self.dirty_regions.push(rect);
    }

    pub fn take_dirty_regions(&mut self) -> Vec<Rect> {
        std::mem::take(&mut self.dirty_regions)
    }
}
