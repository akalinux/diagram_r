use crate::{GridOpt, constants::HALF};

pub struct Grid<'g> {
    grid: &'g GridOpt,
    width: f32,
    height: f32,
    slot: u32,
    x_offset: f32,
    y_offset: f32,
    y_scale: f32,
}

impl<'g> Grid<'g> {
    pub fn new(width: f32, height: f32, grid: &'g GridOpt) -> Self {
        let x_offset = (width % grid.grid_size as f32) * HALF;
        let y_offset = (height % grid.grid_size as f32) * HALF;
        let y_scale = height / width;
        Self {
            grid,
            width,
            height,
            slot: 0,
            x_offset,
            y_offset,
            y_scale,
        }
    }
}

impl<'g> Iterator for Grid<'g> {
    type Item = [f32; 9];
    fn next(&mut self) -> Option<Self::Item> {
        let i = self.slot * self.grid.grid_size;
        if i > self.width as u32 {
            return None;
        }
        self.slot += 1;

        let x1 = i as f32 + self.x_offset;
        let pos = self.slot % self.grid.grid_slots;
        let w = match pos == 0 {
            false => self.grid.grid_divider_width,
            true => self.grid.grid_line_width,
        };
        let x2 = i as f32 * self.y_scale + self.y_offset;
        Some([w, x1, 0.0, x1, self.height, 0.0, x2, self.width, x2])
    }
}
