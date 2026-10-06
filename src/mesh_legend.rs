use egui::{Color32, FontId, Painter, Rect, Stroke, pos2, vec2};

#[derive(Clone, Copy)]
pub enum Marker {
    Dot,
    Ring,
    Diamond,
}

pub struct Entry {
    pub label: &'static str,
    pub color: Color32,
    pub marker: Marker,
}

pub const ENTRIES: [Entry; 7] = [
    Entry {
        label: "Coordinator",
        color: Color32::from_rgb(255, 184, 55),
        marker: Marker::Ring,
    },
    Entry {
        label: "Worker",
        color: Color32::from_rgb(87, 176, 255),
        marker: Marker::Dot,
    },
    Entry {
        label: "Session",
        color: Color32::from_rgb(190, 112, 255),
        marker: Marker::Ring,
    },
    Entry {
        label: "Workspace",
        color: Color32::from_rgb(103, 130, 232),
        marker: Marker::Diamond,
    },
    Entry {
        label: "Working",
        color: Color32::from_rgb(35, 209, 255),
        marker: Marker::Dot,
    },
    Entry {
        label: "Needs input",
        color: Color32::from_rgb(255, 116, 35),
        marker: Marker::Dot,
    },
    Entry {
        label: "Completed",
        color: Color32::from_rgb(67, 227, 121),
        marker: Marker::Dot,
    },
];

pub struct Layout {
    pub bounds: Rect,
    cells: Vec<Rect>,
}

// Keep hierarchy and agent states on separate rows; wrap each group on narrow windows.
pub fn layout(painter: &Painter, viewport: Rect) -> Layout {
    let width = (viewport.width() - 40.0).max(1.0);
    let inner_width = (width - 16.0).max(1.0);
    let mut cells = Vec::with_capacity(ENTRIES.len());
    let (mut x, mut y) = (0.0, 0.0);
    for (i, entry) in ENTRIES.iter().enumerate() {
        let text = painter.layout_no_wrap(entry.label.into(), FontId::monospace(10.0), entry.color);
        let cell_width = (text.size().x + 20.0).min(inner_width);
        if i == 4 || (x > 0.0 && x + cell_width > inner_width) {
            x = 0.0;
            y += 18.0;
        }
        cells.push(Rect::from_min_size(pos2(x, y), vec2(cell_width, 16.0)));
        x += cell_width + 10.0;
    }
    let height = y + 32.0;
    let bounds = Rect::from_min_size(
        pos2(viewport.left() + 20.0, viewport.bottom() - 40.0 - height),
        vec2(width, height),
    );
    for cell in &mut cells {
        *cell = cell.translate(bounds.min.to_vec2() + vec2(8.0, 8.0));
    }
    Layout { bounds, cells }
}

pub fn draw(painter: &Painter, layout: &Layout) {
    let bounds = layout.bounds;
    painter.rect_filled(bounds, 4.0, Color32::from_rgba_unmultiplied(4, 12, 24, 215));
    painter.rect_stroke(
        bounds,
        4.0,
        Stroke::new(0.5, Color32::from_white_alpha(35)),
        egui::StrokeKind::Inside,
    );
    for (entry, cell) in ENTRIES.iter().zip(&layout.cells) {
        let center = cell.left_center() + vec2(5.0, 0.0);
        match entry.marker {
            Marker::Dot => {
                painter.circle_filled(center, 2.8, entry.color);
            }
            Marker::Ring => {
                painter.circle_stroke(center, 4.0, Stroke::new(1.0, entry.color));
                painter.circle_filled(center, 1.5, entry.color);
            }
            Marker::Diamond => {
                painter.add(egui::Shape::closed_line(
                    vec![
                        center + vec2(0.0, -4.0),
                        center + vec2(4.0, 0.0),
                        center + vec2(0.0, 4.0),
                        center + vec2(-4.0, 0.0),
                    ],
                    Stroke::new(1.0, entry.color),
                ));
            }
        }
        painter.text(
            cell.left_center() + vec2(16.0, 0.0),
            egui::Align2::LEFT_CENTER,
            entry.label,
            FontId::monospace(10.0),
            entry.color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_wraps_without_overlapping_or_leaving_the_viewport() {
        let context = egui::Context::default();
        for size in [vec2(280.0, 240.0), vec2(650.0, 460.0), vec2(1200.0, 200.0)] {
            let viewport = Rect::from_min_size(pos2(45.0, 30.0), size);
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(viewport),
                    ..Default::default()
                },
                |ui| {
                    let key = layout(ui.painter(), viewport);
                    let Layout { bounds, cells } = &key;
                    assert!(viewport.contains_rect(*bounds));
                    assert_eq!(cells.len(), ENTRIES.len());
                    for (i, cell) in cells.iter().enumerate() {
                        assert!(bounds.contains_rect(*cell));
                        for other in &cells[i + 1..] {
                            assert!(!cell.intersects(*other));
                        }
                    }
                    assert!(cells[4].top() > cells[3].top());
                    draw(ui.painter(), &key);
                },
            );
            output.textures_delta.clear();
        }
    }
}
