//! 3D constellation geometry and projected, anonymous movie-style callouts.
use crate::logic_core::Viewport;
use crate::mesh_model::{AgentState, Id, PULSE_DURATION, Simulation, ease, noise};
use crate::orbital_sphere::{LineVertex, Particle};
use glam::{Mat4, Quat, Vec3};
use std::f32::consts::{PI, TAU};

pub const MAX_PARTICLES: usize = 60_000;
pub const MAX_LINES: usize = 48_000;
pub struct Geometry {
    pub particles: Vec<Particle>,
    pub lines: Vec<LineVertex>,
}

pub fn camera_distance(viewport: Viewport) -> f32 {
    let aspect = viewport.width / viewport.height.max(1.0);
    let half = (45.0_f32.to_radians() * 0.5).tan();
    3.55 / (half * aspect.clamp(0.01, 1.0)) + 0.5
}
pub fn camera(time: f32, viewport: Viewport) -> (Mat4, Mat4, f32) {
    let aspect = viewport.width / viewport.height.max(1.0);
    let half = (45.0_f32.to_radians() * 0.5).tan();
    let z = camera_distance(viewport);
    let projection =
        glam::camera::rh::proj::directx::perspective(45_f32.to_radians(), aspect, 0.1, 1000.0);
    let view = glam::camera::rh::view::look_at_mat4(Vec3::new(0.0, 0.0, z), Vec3::ZERO, Vec3::Y);
    let model = Mat4::from_scale_rotation_translation(
        Vec3::splat(1.1),
        Quat::from_euler(glam::EulerRot::XYZ, time * 0.018, time * 0.048, 0.0),
        Vec3::ZERO,
    );
    (
        projection * view,
        model,
        (viewport.height * 1.1 / (2.0 * z * half) / 105.0).clamp(0.25, 1.5),
    )
}
fn orbit_rotation(index: usize, time: f32) -> Quat {
    Quat::from_euler(
        glam::EulerRot::XYZ,
        noise(index as u32 + 83) * TAU,
        noise(index as u32 + 23) * TAU,
        time * 0.024 * if index.is_multiple_of(2) { 1.0 } else { -1.0 },
    )
}
fn orbit(index: usize, angle: f32, time: f32) -> Vec3 {
    let radius = 2.5 + noise(index as u32 + 47) * 0.35;
    orbit_rotation(index, time)
        * Vec3::new(
            angle.cos() * radius,
            angle.sin() * radius,
            (angle * 4.0).sin() * 0.1,
        )
}
pub fn coordinator(time: f32) -> Vec3 {
    orbit(5, 0.75 + time * 0.028, time)
}
fn basis(normal: Vec3) -> (Vec3, Vec3) {
    let axis = if normal.y.abs() > 0.95 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let u = normal.cross(axis).normalize();
    (u, normal.cross(u).normalize())
}
fn circle(center: Vec3, radius: f32, angle: f32) -> Vec3 {
    let (u, v) = basis(center.normalize());
    center + radius * (u * angle.cos() + v * angle.sin())
}
// Dyadic angular insertion fills the whole ring for small child counts while
// preserving every earlier slot when children arrive or depart.
fn child_angle(slot: usize) -> f32 {
    ((slot as u32).reverse_bits() as f64 / 4_294_967_296.0) as f32 * TAU
}

pub fn position(id: Id, time: f32) -> Vec3 {
    let (n, s, w, a) = id.indices();
    if matches!(id, Id::Node(..)) {
        return orbit(
            n % 6,
            noise(n as u32 + 131) * TAU + time * (0.025 + noise(n as u32 + 91) * 0.018),
            time,
        );
    }
    let normal = crate::mesh_territory::anchor(n);
    let (u, v) = basis(normal);
    let session = (normal
        + 0.45 * (u * (child_angle(s) + 0.4).cos() + v * (child_angle(s) + 0.4).sin()))
    .normalize()
        * 1.92;
    if matches!(id, Id::Session(..)) {
        return session;
    }
    let workspace = circle(session, 0.42, child_angle(w) + 0.2);
    if matches!(id, Id::Workspace(..)) {
        return workspace;
    }
    circle(workspace, 0.15, child_angle(a) + 0.35)
}
fn visible_position(sim: &Simulation, id: Id) -> Vec3 {
    let pos = position(id, sim.time);
    let alpha = sim
        .entities
        .get(&id)
        .map_or(1.0, |life| life.alpha(sim.time));
    pos * (1.0 + (1.0 - alpha) * 0.15)
}
fn arc(start: Vec3, end: Vec3, t: f32, bulge: f32) -> Vec3 {
    let center = start.lerp(end, t);
    let outward = (start + end).try_normalize().unwrap_or(Vec3::Y);
    center + outward * (PI * t).sin() * bulge
}
impl Geometry {
    fn dot(&mut self, pos: Vec3, size: f32, color: [f32; 3], alpha: f32) {
        self.dot_with_halo(pos, size, color, alpha, 3.4, 0.13);
    }
    fn agent_dot(&mut self, pos: Vec3, color: [f32; 3], alpha: f32) {
        self.dot_with_halo(pos, 2.3, color, alpha, 1.8, 0.045);
    }
    fn dot_with_halo(
        &mut self,
        pos: Vec3,
        size: f32,
        color: [f32; 3],
        alpha: f32,
        halo_size: f32,
        halo_strength: f32,
    ) {
        if alpha <= 0.001 {
            return;
        }
        // Compact agent halos preserve the gap between adjacent status dots.
        self.particles.push(Particle {
            position_size: [pos.x, pos.y, pos.z, size * (0.4 + 0.6 * alpha)],
            color_softness: [color[0] * alpha, color[1] * alpha, color[2] * alpha, 0.92],
        });
        self.particles.push(Particle {
            position_size: [pos.x, pos.y, pos.z, size * halo_size],
            color_softness: [
                color[0] * alpha * halo_strength,
                color[1] * alpha * halo_strength,
                color[2] * alpha * halo_strength,
                0.03,
            ],
        });
    }
    fn line(&mut self, a: Vec3, b: Vec3, color: [f32; 3], alpha: f32) {
        let color = [color[0] * alpha, color[1] * alpha, color[2] * alpha, alpha];
        self.lines.push(LineVertex {
            position: a.to_array(),
            color,
        });
        self.lines.push(LineVertex {
            position: b.to_array(),
            color,
        });
    }
    fn curve(&mut self, a: Vec3, b: Vec3, color: [f32; 3], alpha: f32, bulge: f32) {
        for i in 0..24 {
            self.line(
                arc(a, b, i as f32 / 24.0, bulge),
                arc(a, b, (i + 1) as f32 / 24.0, bulge),
                color,
                alpha,
            );
        }
    }
}
pub fn geometry(sim: &Simulation) -> Geometry {
    let time = sim.time;
    let mut g = Geometry {
        particles: Vec::new(),
        lines: Vec::new(),
    };
    for ring in 0..6 {
        for i in 0..120 {
            g.line(
                orbit(ring, i as f32 * TAU / 120.0, time),
                orbit(ring, (i + 1) as f32 * TAU / 120.0, time),
                [0.21, 0.09, 0.5],
                0.6,
            );
        }
    }
    let root = coordinator(time);
    g.dot(root, 7.0, [1.0, 0.62, 0.12], 1.0);
    // Small crown around the coordinator makes its role distinct from workers.
    for i in 0..32 {
        g.dot(
            circle(root, 0.085, i as f32 * TAU / 32.0 + time * 0.2),
            1.15,
            [1.0, 0.5, 0.08],
            0.7,
        );
    }
    for (&id, life) in &sim.entities {
        let alpha = life.alpha(time);
        if alpha <= 0.001 {
            continue;
        }
        let p = visible_position(sim, id);
        match id {
            Id::Node(_) => {
                g.dot(p, 5.5, [0.25, 0.7, 1.0], alpha);
                g.curve(p, root, [0.3, 0.13, 0.7], alpha * 0.28, 0.5);
            }
            Id::Session(n, _) => {
                g.dot(p, 4.4, [0.7, 0.32, 1.0], alpha);
                g.curve(
                    p,
                    visible_position(sim, Id::Node(n)),
                    [0.25, 0.35, 0.7],
                    alpha * 0.3,
                    0.18,
                );
                for i in 0..24 {
                    g.dot(
                        circle(p, 0.16, i as f32 * TAU / 24.0),
                        0.9,
                        [0.5, 0.15, 0.9],
                        alpha * 0.6,
                    );
                }
            }
            Id::Workspace(n, s, _) => {
                // Hollow diamond makes the workspace distinct from agent dots.
                let diamond =
                    std::array::from_fn::<_, 4, _>(|i| circle(p, 0.065, i as f32 * TAU / 4.0));
                for i in 0..4 {
                    g.line(
                        diamond[i],
                        diamond[(i + 1) % 4],
                        [0.25, 0.5, 1.0],
                        alpha * 0.8,
                    );
                }
                g.dot(p, 1.3, [0.22, 0.35, 0.9], alpha * 0.6);
                g.line(
                    p,
                    visible_position(sim, Id::Session(n, s)),
                    [0.25, 0.2, 0.6],
                    alpha * 0.3,
                );
            }
            Id::Agent(..) => {
                let state = sim.state(id);
                let breathing = match state {
                    AgentState::Working => {
                        0.75 + 0.25 * (time * 2.8 + noise(id.seed()) * TAU).sin()
                    }
                    AgentState::Blocked => 0.85 + 0.15 * (time * 0.9).sin(),
                    AgentState::Completed => 0.6,
                };
                let (n, s, w, _) = id.indices();
                g.line(
                    visible_position(sim, Id::Workspace(n, s, w)),
                    p,
                    [0.12, 0.3, 0.45],
                    alpha * 0.22,
                );
                g.agent_dot(p, state.color(), alpha * breathing);
            }
        }
    }
    for pulse in &sim.pulses {
        let age = (time - pulse.started) / PULSE_DURATION;
        let (n, s, w, _) = pulse.origin.indices();
        let start = visible_position(sim, pulse.origin);
        let session = visible_position(sim, Id::Session(n, s));
        let node = visible_position(sim, Id::Node(n));
        // Leaf -> workspace -> session ripple, then session -> worker -> coordinator.
        let first_end = if matches!(pulse.origin, Id::Node(..)) {
            node
        } else {
            session
        };
        let p = if age < 0.35 {
            let local = ease(age / 0.35);
            let workspace = visible_position(sim, Id::Workspace(n, s, w));
            let ripple_center = if matches!(pulse.origin, Id::Agent(..)) {
                start
                    .lerp(workspace, local.min(0.5) * 2.0)
                    .lerp(session, (local - 0.5).max(0.0) * 2.0)
            } else {
                start.lerp(first_end, local)
            };
            for i in 0..36 {
                g.dot(
                    circle(ripple_center, 0.03 + local * 0.22, i as f32 * TAU / 36.0),
                    1.4,
                    pulse.color,
                    (1.0 - local) * 0.7,
                );
            }
            ripple_center
        } else if age < 0.55 {
            arc(first_end, node, ease((age - 0.35) / 0.2), 0.18)
        } else {
            arc(node, root, ease((age - 0.55) / 0.45), 0.5)
        };
        g.dot(p, 5.0, pulse.color, (PI * age).sin().max(0.0));
        if age > 0.55 {
            let head = ease((age - 0.55) / 0.45);
            for i in 1..12 {
                let t = (head - i as f32 * 0.012).max(0.0);
                g.dot(
                    arc(node, root, t, 0.5),
                    2.5,
                    pulse.color,
                    (1.0 - i as f32 / 12.0) * 0.45,
                );
            }
        }
    }
    debug_assert!(g.particles.len() <= MAX_PARTICLES);
    debug_assert!(g.lines.len() <= MAX_LINES);
    g
}
fn project(pos: Vec3, time: f32, rect: egui::Rect) -> Option<egui::Pos2> {
    let (vp, model, _) = camera(
        time,
        Viewport {
            x: rect.left(),
            y: rect.top(),
            width: rect.width(),
            height: rect.height(),
        },
    );
    let clip = vp * model * pos.extend(1.0);
    if clip.w <= 0.0 {
        return None;
    }
    let p = clip.truncate() / clip.w;
    Some(egui::pos2(
        rect.center().x + p.x * rect.width() * 0.5,
        rect.center().y - p.y * rect.height() * 0.5,
    ))
}
pub fn overlay(ui: &egui::Ui, sim: &Simulation, rect: egui::Rect) {
    let painter = ui.painter().with_clip_rect(rect);
    let cyan = egui::Color32::from_rgb(90, 216, 235);
    painter.text(
        rect.left_top() + egui::vec2(20.0, 18.0),
        egui::Align2::LEFT_TOP,
        "MESH ORB  /  SYNTHETIC TELEMETRY",
        egui::FontId::monospace(12.0),
        cyan,
    );
    let c = sim.counts();
    painter.text(
        rect.left_bottom() + egui::vec2(20.0, -18.0),
        egui::Align2::LEFT_BOTTOM,
        format!(
            "{} SATELLITES   /   {} ACTIVE   /   {} BLOCKED   /   {} COMPLETE",
            sim.settings.nodes, c[0], c[1], c[2]
        ),
        egui::FontId::monospace(11.0),
        cyan,
    );
    if let Some(anchor) = project(coordinator(sim.time), sim.time, rect) {
        painter.circle_stroke(
            anchor,
            14.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(242, 177, 70)),
        );
    }
    if !sim.callouts || rect.width() < 260.0 || rect.height() < 200.0 {
        return;
    }
    for (slot, event) in sim.events.iter().rev().enumerate() {
        let age = sim.time - event.started;
        let reveal = ease(age / 0.65);
        let opacity = reveal * (1.0 - ease((age - 4.8) / 1.2));
        let col = egui::Color32::from_rgb(
            (event.color[0] * 255.0) as u8,
            (event.color[1] * 255.0) as u8,
            (event.color[2] * 255.0) as u8,
        )
        .gamma_multiply(opacity);
        let anchor =
            project(visible_position(sim, event.origin), sim.time, rect).unwrap_or(rect.center());
        let width = 240.0_f32.min(rect.width() * 0.38).max(120.0);
        // HUD cards drift gently at the edges while their leaders track real 3D anchors.
        let x = if slot == 1 {
            rect.left() + 18.0
        } else {
            rect.right() - width - 18.0
        };
        let y = rect.top()
            + 55.0
            + slot as f32 * (rect.height() - 150.0).max(0.0) / 3.0
            + (sim.time * 0.24 + slot as f32).sin() * 9.0;
        let card = egui::Rect::from_min_size(
            egui::pos2(x + (1.0 - reveal) * 22.0, y),
            egui::vec2(width, 84.0),
        );
        painter.rect_filled(
            card,
            0.0,
            egui::Color32::from_rgba_unmultiplied(4, 12, 24, (205.0 * opacity) as u8),
        );
        painter.rect_stroke(
            card,
            0.0,
            egui::Stroke::new(0.7, col.gamma_multiply(0.45)),
            egui::StrokeKind::Inside,
        );
        for (a, b) in [
            (card.left_top(), card.left_top() + egui::vec2(16.0, 0.0)),
            (card.left_top(), card.left_top() + egui::vec2(0.0, 10.0)),
            (
                card.right_bottom(),
                card.right_bottom() - egui::vec2(16.0, 0.0),
            ),
            (
                card.right_bottom(),
                card.right_bottom() - egui::vec2(0.0, 10.0),
            ),
        ] {
            painter.line_segment([a, b], egui::Stroke::new(1.5, col));
        }
        let end = if slot == 1 {
            card.right_center()
        } else {
            card.left_center()
        };
        let elbow = egui::pos2((anchor.x + end.x) * 0.5, end.y);
        painter.add(egui::Shape::line(
            vec![anchor, elbow, end],
            egui::Stroke::new(0.8, col.gamma_multiply(0.6)),
        ));
        painter.circle_stroke(
            anchor,
            4.0 + 3.0 * (1.0 - reveal),
            egui::Stroke::new(1.0, col),
        );
        let text = event
            .title
            .chars()
            .take((age * 36.0) as usize)
            .collect::<String>();
        painter.text(
            card.left_top() + egui::vec2(12.0, 12.0),
            egui::Align2::LEFT_TOP,
            text,
            egui::FontId::monospace(11.0),
            col,
        );
        let galley = painter.layout(
            event.text.clone(),
            egui::FontId::monospace(10.0),
            col.gamma_multiply(0.8),
            width - 24.0,
        );
        painter.galley(card.left_top() + egui::vec2(12.0, 32.0), galley, col);
        painter.line_segment(
            [
                card.left_bottom() + egui::vec2(12.0, -10.0),
                card.left_bottom() + egui::vec2(12.0 + (width - 24.0) * reveal, -10.0),
            ],
            egui::Stroke::new(1.0, col),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh_model::{MAX_AGENTS, MAX_NODES, MAX_SESSIONS, MAX_WORKSPACES, Settings};
    #[test]
    fn small_agent_rings_are_distributed_and_readable_in_the_default_view() {
        let workspace = position(Id::Workspace(0, 0, 0), 0.0);
        let agents = (0..6)
            .map(|a| position(Id::Agent(0, 0, 0, a), 0.0))
            .collect::<Vec<_>>();
        // Six occupied slots span the whole circle, instead of one short arc.
        let center = agents.iter().copied().sum::<Vec3>() / agents.len() as f32;
        assert!((center - workspace).length() < 1e-5);
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(640.0, 460.0));
        for (i, a) in agents.iter().enumerate() {
            for b in &agents[i + 1..] {
                assert!(
                    project(*a, 0.0, rect)
                        .unwrap()
                        .distance(project(*b, 0.0, rect).unwrap())
                        > 4.0
                );
            }
        }
        // Different workspaces/sessions have breathing room around their leaves.
        assert!(
            position(Id::Workspace(0, 0, 0), 0.0).distance(position(Id::Workspace(0, 0, 2), 0.0))
                > 0.59
        );
        assert!(position(Id::Session(0, 0), 0.0).distance(position(Id::Session(0, 1), 0.0)) > 1.5);
    }

    #[test]
    fn maximum_scene_and_departures_fit_gpu_buffers() {
        let mut s = Simulation::default();
        s.configure(Settings {
            nodes: MAX_NODES,
            sessions: MAX_SESSIONS,
            workspaces: MAX_WORKSPACES,
            agents: MAX_AGENTS,
            ..s.settings
        });
        for _ in 0..30 {
            s.advance(0.1);
        }
        s.trigger();
        let g = geometry(&s);
        assert!(g.particles.len() <= MAX_PARTICLES);
        assert!(g.lines.len() <= MAX_LINES);
        assert!(
            g.particles
                .iter()
                .all(|p| p.position_size.iter().all(|n| n.is_finite()))
        );
        s.configure(Settings {
            nodes: 0,
            ..s.settings
        });
        assert!(!geometry(&s).particles.is_empty());
    }
    #[test]
    fn placement_stays_stable_as_counts_change_and_projects_resize() {
        let id = Id::Agent(2, 1, 2, 3);
        let p = position(id, 2.0);
        // Surface placement does not follow a worker's changing orbital angle.
        assert_eq!(p, position(id, 120.0));
        assert_ne!(position(Id::Node(2), 2.0), position(Id::Node(2), 120.0));
        let mut sim = Simulation::default();
        sim.configure(Settings {
            nodes: MAX_NODES,
            ..sim.settings
        });
        assert!(
            (p - visible_position(&sim, id)
                / (1.0 + (1.0 - sim.entities[&id].alpha(sim.time)) * 0.15))
                .length()
                < 1e-5
        );
        for (w, h) in [(1440.0, 900.0), (500.0, 900.0), (900.0, 200.0)] {
            let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(w, h));
            assert!(rect.contains(project(coordinator(2.0), 2.0, rect).unwrap()));
        }
    }
}
