use egui::{
    Align, Align2, Color32, CornerRadius, FontId, Layout, Pos2, Rect, RichText, Sense, Stroke,
    StrokeKind, Vec2,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Experience {
    #[default]
    Cube,
    LogicCore,
}

#[derive(Default)]
pub struct UiLayout {
    pub logic_viewport: Option<Rect>,
}

pub fn draw(root_ui: &mut egui::Ui, experience: &mut Experience, elapsed: f32) -> UiLayout {
    let mut layout = UiLayout::default();
    egui::Panel::top("experience_selector")
        .frame(
            egui::Frame::new()
                .fill(Color32::from_rgba_unmultiplied(5, 5, 5, 238))
                .inner_margin(egui::Margin::symmetric(18, 12))
                .stroke(Stroke::new(1.0, Color32::from_white_alpha(18))),
        )
        .show(root_ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("WEBGPU EXPERIENCES")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_gray(175)),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    selector_button(ui, experience, Experience::LogicCore, "Logic Core", "2");
                    selector_button(ui, experience, Experience::Cube, "Cube", "1");
                });
            });
        });

    if *experience == Experience::LogicCore {
        layout.logic_viewport = Some(draw_logic_core_ui(root_ui, elapsed));
    } else {
        egui::Area::new("cube_help".into())
            .anchor(Align2::LEFT_BOTTOM, Vec2::new(24.0, -22.0))
            .show(root_ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(Color32::from_rgba_unmultiplied(8, 10, 16, 210))
                    .corner_radius(12)
                    .inner_margin(12)
                    .stroke(Stroke::new(1.0, Color32::from_white_alpha(24)))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("DRAG TO ROTATE  ·  WASD / ARROWS")
                                .monospace()
                                .size(11.0)
                                .color(Color32::from_gray(180)),
                        );
                    });
            });
    }

    layout
}

fn selector_button(
    ui: &mut egui::Ui,
    selected: &mut Experience,
    value: Experience,
    label: &str,
    shortcut: &str,
) {
    let active = *selected == value;
    let text = RichText::new(format!("{label}   {shortcut}"))
        .size(13.0)
        .color(if active {
            Color32::from_rgb(215, 252, 255)
        } else {
            Color32::from_gray(150)
        });
    let button = egui::Button::new(text)
        .fill(if active {
            Color32::from_rgb(8, 54, 60)
        } else {
            Color32::from_rgb(15, 15, 15)
        })
        .stroke(Stroke::new(
            1.0,
            if active {
                Color32::from_rgb(0, 180, 200)
            } else {
                Color32::from_white_alpha(18)
            },
        ))
        .corner_radius(9);
    if ui.add(button).clicked() {
        *selected = value;
    }
}

fn draw_logic_core_ui(root_ui: &mut egui::Ui, elapsed: f32) -> Rect {
    let mut logic_viewport = Rect::NOTHING;
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(Color32::TRANSPARENT))
        .show(root_ui, |ui| {
            let available = ui.available_rect_before_wrap();
            draw_grid(ui, available);

            let content_width = available.width().min(1040.0);
            let gap = 28.0;
            let card_width = ((content_width - gap) * 0.5).max(280.0);
            let card_height = available.height().clamp(430.0, 540.0);
            let start = Pos2::new(
                available.center().x - content_width * 0.5,
                available.center().y - card_height * 0.5 + 10.0,
            );
            let left = Rect::from_min_size(start, Vec2::new(card_width, card_height));
            let right = Rect::from_min_size(
                Pos2::new(start.x + card_width + gap, start.y),
                Vec2::new(card_width, card_height),
            );

            draw_card_shell(ui, left);
            draw_logic_card_shell(ui, right);
            draw_collaboration_visual(ui, left, elapsed);
            logic_viewport = draw_logic_card_overlay(ui, right, elapsed);
        });
    logic_viewport
}

fn draw_grid(ui: &egui::Ui, rect: Rect) {
    let painter = ui.painter();
    let color = Color32::from_white_alpha(9);
    let spacing = 40.0;
    let mut x = rect.left() - rect.left().rem_euclid(spacing);
    while x < rect.right() {
        painter.line_segment(
            [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
            Stroke::new(1.0, color),
        );
        x += spacing;
    }
    let mut y = rect.top() - rect.top().rem_euclid(spacing);
    while y < rect.bottom() {
        painter.line_segment(
            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
            Stroke::new(1.0, color),
        );
        y += spacing;
    }
}

fn draw_card_shell(ui: &egui::Ui, rect: Rect) {
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(24), Color32::from_rgb(10, 10, 10));
    painter.rect_stroke(
        rect,
        CornerRadius::same(24),
        Stroke::new(1.0, Color32::from_white_alpha(22)),
        StrokeKind::Inside,
    );
}

fn draw_logic_card_shell(ui: &egui::Ui, rect: Rect) {
    let painter = ui.painter();
    let body = Rect::from_min_max(Pos2::new(rect.left(), rect.top() + 300.0), rect.max);
    painter.rect_filled(
        body,
        CornerRadius {
            nw: 0,
            ne: 0,
            sw: 24,
            se: 24,
        },
        Color32::from_rgb(10, 10, 10),
    );
    painter.rect_stroke(
        rect,
        CornerRadius::same(24),
        Stroke::new(1.0, Color32::from_white_alpha(22)),
        StrokeKind::Inside,
    );
}

fn draw_collaboration_visual(ui: &egui::Ui, card: Rect, elapsed: f32) {
    let painter = ui.painter();
    let visual = Rect::from_min_max(card.min, Pos2::new(card.right(), card.top() + 300.0));
    painter.rect_filled(
        visual,
        CornerRadius {
            nw: 24,
            ne: 24,
            sw: 0,
            se: 0,
        },
        Color32::from_rgb(10, 10, 10),
    );
    painter.line_segment(
        [
            Pos2::new(visual.left(), visual.bottom()),
            Pos2::new(visual.right(), visual.bottom()),
        ],
        Stroke::new(1.0, Color32::from_white_alpha(14)),
    );

    let center = visual.center();
    for (radius, alpha) in [(126.0, 8), (88.0, 13), (50.0, 22)] {
        painter.circle_stroke(
            center,
            radius,
            Stroke::new(1.0, Color32::from_white_alpha(alpha)),
        );
    }
    let selection = Rect::from_center_size(center, Vec2::new(120.0, 60.0));
    painter.rect_stroke(
        selection,
        0,
        Stroke::new(1.0, Color32::from_rgb(79, 70, 229)),
        StrokeKind::Inside,
    );
    for point in [
        selection.left_top(),
        selection.right_top(),
        selection.left_bottom(),
        selection.right_bottom(),
    ] {
        painter.rect_filled(
            Rect::from_center_size(point, Vec2::splat(5.0)),
            0,
            Color32::from_rgb(79, 70, 229),
        );
    }

    avatar(
        painter,
        center + Vec2::new(-78.0, -70.0),
        Color32::from_rgb(90, 116, 142),
    );
    avatar(
        painter,
        center + Vec2::new(104.0, 42.0),
        Color32::from_rgb(98, 83, 76),
    );
    avatar(
        painter,
        center + Vec2::new(-112.0, 76.0),
        Color32::from_rgb(118, 91, 105),
    );
    collaborator(
        painter,
        center + Vec2::new(-90.0 + (elapsed * 1.2).sin() * 6.0, -60.0),
        "Alex",
        Color32::from_rgb(249, 115, 22),
    );
    collaborator(
        painter,
        center + Vec2::new(-70.0, 52.0 + (elapsed * 1.1).sin() * 5.0),
        "Sam",
        Color32::from_rgb(236, 72, 153),
    );
    collaborator(
        painter,
        center + Vec2::new(72.0, 34.0 + (elapsed * 1.3).cos() * 7.0),
        "Me",
        Color32::from_rgb(16, 185, 129),
    );

    draw_text_block(
        painter,
        card,
        "Visual Interface",
        "Construct dynamic layouts using our intuitive\nnode-based system. Collaborate in real-time\nacross unified environments.",
    );
}

fn avatar(painter: &egui::Painter, center: Pos2, color: Color32) {
    painter.circle_filled(center, 11.0, Color32::from_rgb(20, 20, 20));
    painter.circle_filled(center, 9.0, color);
    painter.circle_filled(center + Vec2::new(0.0, -2.5), 3.0, Color32::from_gray(205));
    painter.circle_filled(center + Vec2::new(0.0, 5.0), 5.0, Color32::from_gray(150));
}

fn collaborator(painter: &egui::Painter, position: Pos2, name: &str, color: Color32) {
    let points = vec![
        position,
        position + Vec2::new(12.0, 12.0),
        position + Vec2::new(5.0, 12.0),
        position + Vec2::new(0.0, 18.0),
    ];
    painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
    let pill = Rect::from_min_size(position + Vec2::new(10.0, -9.0), Vec2::new(42.0, 19.0));
    painter.rect_filled(pill, 10, color);
    painter.text(
        pill.center(),
        Align2::CENTER_CENTER,
        name,
        FontId::proportional(10.0),
        Color32::WHITE,
    );
}

fn draw_logic_card_overlay(ui: &mut egui::Ui, card: Rect, elapsed: f32) -> Rect {
    let painter = ui.painter();
    let viewport = Rect::from_min_max(
        card.min + Vec2::new(1.0, 1.0),
        Pos2::new(card.right() - 1.0, card.top() + 299.0),
    );
    painter.line_segment(
        [
            Pos2::new(card.left(), viewport.bottom()),
            Pos2::new(card.right(), viewport.bottom()),
        ],
        Stroke::new(1.0, Color32::from_white_alpha(14)),
    );

    let terminal = Rect::from_center_size(viewport.center(), Vec2::new(96.0, 96.0));
    painter.rect_filled(
        terminal,
        CornerRadius::same(18),
        Color32::from_rgba_unmultiplied(15, 15, 15, 232),
    );
    painter.rect_stroke(
        terminal,
        CornerRadius::same(18),
        Stroke::new(1.0, Color32::from_white_alpha(26)),
        StrokeKind::Inside,
    );
    painter.text(
        terminal.center() + Vec2::new(-9.0, -1.0),
        Align2::CENTER_CENTER,
        ">",
        FontId::monospace(31.0),
        Color32::from_rgb(34, 211, 238),
    );
    if (elapsed * 2.0).fract() < 0.55 {
        painter.rect_filled(
            Rect::from_center_size(
                terminal.center() + Vec2::new(16.0, 8.0),
                Vec2::new(16.0, 2.0),
            ),
            0,
            Color32::from_rgb(34, 211, 238),
        );
    }

    draw_text_block(
        painter,
        card,
        "Core Logic",
        "Access the underlying isometric data structures\ndirectly. Deploy scalable services through a\nhigh-performance runtime.",
    );
    let interaction = ui.interact(viewport, ui.id().with("logic_core_view"), Sense::hover());
    if interaction.hovered() {
        painter.rect_stroke(
            viewport.shrink(2.0),
            20,
            Stroke::new(1.0, Color32::from_rgb(0, 130, 150)),
            StrokeKind::Inside,
        );
    }
    viewport
}

fn draw_text_block(painter: &egui::Painter, card: Rect, title: &str, body: &str) {
    let title_pos = Pos2::new(card.left() + 28.0, card.top() + 334.0);
    painter.text(
        title_pos,
        Align2::LEFT_TOP,
        title,
        FontId::proportional(22.0),
        Color32::from_gray(242),
    );
    painter.text(
        title_pos + Vec2::new(0.0, 43.0),
        Align2::LEFT_TOP,
        body,
        FontId::proportional(14.0),
        Color32::from_gray(150),
    );
}
