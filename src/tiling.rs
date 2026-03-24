use ratatui::layout::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LayoutMode {
    MasterStack,
    Monocle,
    Grid,
}

impl LayoutMode {
    pub fn name(&self) -> &'static str {
        match self {
            LayoutMode::MasterStack => "[]= ",
            LayoutMode::Monocle => "[M]",
            LayoutMode::Grid => "[#]",
        }
    }
}

pub fn tile(
    area: Rect,
    n: usize,
    focused: usize,
    layout: LayoutMode,
    master_count: usize,
    master_ratio: f64,
) -> Vec<Rect> {
    if n == 0 {
        return vec![];
    }
    match layout {
        LayoutMode::MasterStack => tile_master_stack(area, n, master_count, master_ratio),
        LayoutMode::Monocle => tile_monocle(area, focused, n),
        LayoutMode::Grid => tile_grid(area, n),
    }
}

fn tile_master_stack(
    area: Rect,
    n: usize,
    master_count: usize,
    master_ratio: f64,
) -> Vec<Rect> {
    let mut rects = Vec::with_capacity(n);
    let mc = master_count.min(n);

    if n <= mc {
        // All panes are master — split vertically into n equal parts
        let h = area.height / n as u16;
        for i in 0..n {
            let y = area.y + (i as u16) * h;
            let height = if i == n - 1 {
                area.height - (i as u16) * h
            } else {
                h
            };
            rects.push(Rect::new(area.x, y, area.width, height));
        }
    } else {
        // Split horizontally: master on left, stack on right
        let master_w = (area.width as f64 * master_ratio) as u16;
        let stack_w = area.width - master_w;
        let stack_n = n - mc;

        // Master panes
        let mh = area.height / mc as u16;
        for i in 0..mc {
            let y = area.y + (i as u16) * mh;
            let height = if i == mc - 1 {
                area.height - (i as u16) * mh
            } else {
                mh
            };
            rects.push(Rect::new(area.x, y, master_w, height));
        }

        // Stack panes
        let sh = area.height / stack_n as u16;
        for i in 0..stack_n {
            let y = area.y + (i as u16) * sh;
            let height = if i == stack_n - 1 {
                area.height - (i as u16) * sh
            } else {
                sh
            };
            rects.push(Rect::new(area.x + master_w, y, stack_w, height));
        }
    }

    rects
}

fn tile_monocle(area: Rect, focused: usize, n: usize) -> Vec<Rect> {
    // Return only the focused pane at full size
    // We return n rects but only the focused one gets the full area,
    // others get zero-size (they won't be rendered)
    let mut rects = Vec::with_capacity(n);
    for i in 0..n {
        if i == focused {
            rects.push(area);
        } else {
            rects.push(Rect::new(0, 0, 0, 0));
        }
    }
    rects
}

fn tile_grid(area: Rect, n: usize) -> Vec<Rect> {
    let cols = (n as f64).sqrt().ceil() as usize;
    let rows = (n + cols - 1) / cols;

    let col_w = area.width / cols as u16;
    let row_h = area.height / rows as u16;

    let mut rects = Vec::with_capacity(n);
    for i in 0..n {
        let r = i / cols;
        let c = i % cols;

        // Items in the last row
        let items_in_row = if r == rows - 1 {
            n - r * cols
        } else {
            cols
        };

        let w = if r == rows - 1 {
            // Last row: stretch to fill
            let w = area.width / items_in_row as u16;
            if c == items_in_row - 1 {
                area.width - (c as u16) * w
            } else {
                w
            }
        } else if c == cols - 1 {
            area.width - (c as u16) * col_w
        } else {
            col_w
        };

        let h = if r == rows - 1 {
            area.height - (r as u16) * row_h
        } else {
            row_h
        };

        let x = if r == rows - 1 {
            let row_w = area.width / items_in_row as u16;
            area.x + (c as u16) * row_w
        } else {
            area.x + (c as u16) * col_w
        };
        let y = area.y + (r as u16) * row_h;

        rects.push(Rect::new(x, y, w, h));
    }

    rects
}
