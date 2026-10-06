use crate::mesh_model::{
    MAX_AGENTS, MAX_NODES, MAX_SESSIONS, MAX_WORKSPACES, Settings, Simulation,
};
use egui::{Color32, RichText};

pub fn draw(root: &mut egui::Ui, sim: &mut Simulation) {
    if !sim.controls {
        return;
    }
    egui::Panel::left("mesh_lab")
        .default_size(268.0)
        .resizable(true)
        .size_range(220.0..=360.0)
        .frame(egui::Frame::new().fill(Color32::from_rgb(6,13,24)).inner_margin(18))
        .show(root,|ui| {
            egui::ScrollArea::vertical().show(ui,|ui| {
                ui.label(RichText::new("CONSTELLATION LAB").monospace().size(16.0).color(Color32::from_rgb(104,220,242)));
                ui.label("Phase 1 · synthetic data"); ui.separator();
                let mut s=sim.settings;
                ui.label("Topology");
                ui.add(egui::Slider::new(&mut s.nodes,0..=MAX_NODES).text("nodes"));
                ui.add(egui::Slider::new(&mut s.sessions,0..=MAX_SESSIONS).text("sessions / node"));
                ui.add(egui::Slider::new(&mut s.workspaces,0..=MAX_WORKSPACES).text("spaces / session"));
                ui.add(egui::Slider::new(&mut s.agents,0..=MAX_AGENTS).text("agents / space"));
                ui.separator(); ui.label("Agent distribution");
                ui.add(egui::Slider::new(&mut s.working,0..=100).text("working %"));
                s.blocked=s.blocked.min(100-s.working);
                ui.add(egui::Slider::new(&mut s.blocked,0..=100-s.working).text("blocked %"));
                ui.label(format!("{}% completed",100-s.working-s.blocked));
                sim.configure(s);
                ui.separator(); ui.label("Playback");
                ui.checkbox(&mut sim.running,"Run animation");
                ui.checkbox(&mut sim.auto_activity,"Simulate state changes");
                ui.checkbox(&mut sim.callouts,"Animated callouts");
                ui.add(egui::Slider::new(&mut sim.speed,0.1..=3.0).text("time scale"));
                ui.add(egui::Slider::new(&mut sim.period,3.0..=15.0).text("event interval"));
                if ui.button("Trigger activity / pulse").clicked() {sim.trigger();}
                ui.separator(); ui.label("Scenarios");
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Quiet").clicked() { sim.auto_activity=false;sim.configure(Settings {nodes:3,sessions:1,workspaces:2,agents:4,working:0,blocked:0}); }
                    if ui.button("Busy").clicked() {sim.auto_activity=true;sim.configure(Settings::default());}
                    if ui.button("Blocked").clicked() {sim.auto_activity=false;sim.configure(Settings {working:15,blocked:75,..sim.settings});}
                    if ui.button("Dense").clicked() {sim.configure(Settings {nodes:18,sessions:3,workspaces:6,agents:12,..Settings::default()});}
                    if ui.button("Empty").clicked() {sim.configure(Settings {nodes:0,..sim.settings});}
                });
                ui.separator();
                let c=sim.counts();
                ui.label(format!("{} sessions · {} workspaces\n{} agents",sim.settings.nodes*sim.settings.sessions,sim.settings.nodes*sim.settings.sessions*sim.settings.workspaces,c.iter().sum::<usize>()));
                for entry in &crate::mesh_legend::ENTRIES {
                    ui.horizontal(|ui| {
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(36.0, 32.0), egui::Sense::hover());
                        crate::mesh_legend::sample(ui.painter(), entry.glyph, rect, sim.time);
                        ui.colored_label(entry.color(), entry.label);
                    });
                }
                ui.separator(); ui.small("H hides this panel. Space pauses. P triggers activity.\nCounts reshape the scene over 1.2 s; pulses travel leaf → session → node → coordinator in 3 s.");
            });
        });
}
