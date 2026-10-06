use std::sync::Arc;

use egui::{Color32, FontId, Galley, Painter, Pos2, Rect, pos2, vec2};

use crate::mesh_model::Summary;

const COLOR: Color32 = Color32::from_rgb(90, 216, 235);

pub struct Layout {
    pub bounds: Rect,
    items: Vec<(Pos2, Arc<Galley>)>,
}

// Keep each count and label together, wrapping whole entries above the bottom margin.
pub fn layout(painter: &Painter, viewport: Rect, summary: Summary) -> Layout {
    let margin = (viewport.width() * 0.05).min(20.0);
    let width = (viewport.width() - 2.0 * margin).max(1.0);
    let values = [
        (summary.nodes, "NODES"),
        (summary.sessions, "SESSIONS"),
        (summary.workspaces, "WORKSPACES"),
        (summary.agents, "AGENTS"),
        (summary.states[0], "WORKING"),
        (summary.states[1], "BLOCKED"),
        (summary.states[2], "COMPLETE"),
    ];
    let mut items = Vec::with_capacity(values.len());
    let (mut x, mut y, mut row_height) = (0.0_f32, 0.0_f32, 0.0_f32);
    for (count, label) in values {
        let text = format!("{count} {label}");
        let mut galley = painter.layout_no_wrap(text.clone(), FontId::monospace(11.0), COLOR);
        // Very narrow views still keep a complete entry inside the viewport.
        if galley.size().x > width {
            let font_size = 11.0 * width / galley.size().x;
            galley = painter.layout_no_wrap(text, FontId::monospace(font_size), COLOR);
        }
        if x > 0.0 && x + galley.size().x > width {
            x = 0.0;
            y += row_height + 6.0;
            row_height = 0.0;
        }
        items.push((pos2(x, y), galley.clone()));
        row_height = row_height.max(galley.size().y);
        x += galley.size().x + 18.0;
    }
    let height = y + row_height;
    let bounds = Rect::from_min_size(
        pos2(viewport.left() + margin, viewport.bottom() - 18.0 - height),
        vec2(width, height),
    );
    for (position, _) in &mut items {
        *position += bounds.min.to_vec2();
    }
    Layout { bounds, items }
}

pub fn draw(painter: &Painter, layout: &Layout) {
    for (position, galley) in &layout.items {
        painter.galley(*position, galley.clone(), COLOR);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh_model::Simulation;

    #[test]
    fn totals_wrap_as_whole_entries_and_reserve_space_for_the_key() {
        let context = egui::Context::default();
        let summary = Simulation::default().summary();
        for size in [vec2(320.0, 480.0), vec2(650.0, 460.0), vec2(1200.0, 200.0)] {
            let viewport = Rect::from_min_size(pos2(45.0, 30.0), size);
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(viewport),
                    ..Default::default()
                },
                |ui| {
                    let stats = layout(ui.painter(), viewport, summary);
                    assert!(viewport.contains_rect(stats.bounds));
                    assert_eq!(stats.items.len(), 7);
                    let key = crate::mesh_legend::layout(
                        ui.painter(),
                        viewport,
                        stats.bounds.top() - 10.0,
                    );
                    assert!(key.bounds.bottom() <= stats.bounds.top() - 10.0);
                    for (i, (position, galley)) in stats.items.iter().enumerate() {
                        let cell = Rect::from_min_size(*position, galley.size());
                        assert!(stats.bounds.expand(0.01).contains_rect(cell));
                        for (other_position, other) in &stats.items[i + 1..] {
                            assert!(
                                !cell
                                    .intersects(Rect::from_min_size(*other_position, other.size()))
                            );
                        }
                    }
                    assert_eq!(stats.items[3].1.text(), "216 AGENTS");
                    if size.x < 400.0 {
                        assert!(stats.items.last().unwrap().0.y > stats.items[0].0.y);
                    }
                    draw(ui.painter(), &stats);
                },
            );
            output.textures_delta.clear();
        }
    }
}
