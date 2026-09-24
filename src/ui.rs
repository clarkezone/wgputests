use egui::{Align, Align2, Color32, Layout, Rect, RichText, Stroke, Vec2};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Experience {
    #[default]
    Cube,
    LogicCore,
    OrbitalSphere,
}

#[derive(Default)]
pub struct UiLayout {
    pub logic_viewport: Option<Rect>,
}

pub fn draw(root_ui: &mut egui::Ui, experience: &mut Experience, _elapsed: f32) -> UiLayout {
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
                    selector_button(
                        ui,
                        experience,
                        Experience::OrbitalSphere,
                        "Orbital Sphere",
                        "3",
                    );
                    selector_button(ui, experience, Experience::LogicCore, "Logic Core", "2");
                    selector_button(ui, experience, Experience::Cube, "Cube", "1");
                });
            });
        });

    if matches!(
        *experience,
        Experience::LogicCore | Experience::OrbitalSphere
    ) {
        let mut viewport = Rect::NOTHING;
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::TRANSPARENT))
            .show(root_ui, |ui| viewport = ui.available_rect_before_wrap());
        UiLayout {
            logic_viewport: Some(viewport),
        }
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
        UiLayout::default()
    }
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
