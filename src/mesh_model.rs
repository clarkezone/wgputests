//! Synthetic-only replacement model. No networking or Herdr dependency.
use std::collections::{BTreeMap, VecDeque};

pub const TRANSITION: f32 = 1.2;
pub const PULSE_DURATION: f32 = 3.0;
pub const MAX_NODES: usize = 24;
pub const MAX_SESSIONS: usize = 4;
pub const MAX_WORKSPACES: usize = 8;
pub const MAX_AGENTS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub nodes: usize,
    pub sessions: usize,
    pub workspaces: usize,
    pub agents: usize,
    pub working: u32,
    pub blocked: u32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            nodes: 6,
            sessions: 2,
            workspaces: 3,
            agents: 6,
            working: 60,
            blocked: 15,
        }
    }
}
impl Settings {
    pub fn bounded(mut self) -> Self {
        self.nodes = self.nodes.min(MAX_NODES);
        self.sessions = self.sessions.min(MAX_SESSIONS);
        self.workspaces = self.workspaces.min(MAX_WORKSPACES);
        self.agents = self.agents.min(MAX_AGENTS);
        self.working = self.working.min(100);
        self.blocked = self.blocked.min(100 - self.working);
        self
    }
    pub fn contains(self, id: Id) -> bool {
        let (n, s, w, a) = id.indices();
        n < self.nodes
            && match id {
                Id::Node(_) => true,
                Id::Session(..) => s < self.sessions,
                Id::Workspace(..) => s < self.sessions && w < self.workspaces,
                Id::Agent(..) => s < self.sessions && w < self.workspaces && a < self.agents,
            }
    }
    fn ids(self) -> impl Iterator<Item = Id> {
        let mut ids = Vec::new();
        for n in 0..self.nodes {
            ids.push(Id::Node(n));
            for s in 0..self.sessions {
                ids.push(Id::Session(n, s));
                for w in 0..self.workspaces {
                    ids.push(Id::Workspace(n, s, w));
                    for a in 0..self.agents {
                        ids.push(Id::Agent(n, s, w, a));
                    }
                }
            }
        }
        ids.into_iter()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Id {
    Node(usize),
    Session(usize, usize),
    Workspace(usize, usize, usize),
    Agent(usize, usize, usize, usize),
}
impl Id {
    pub fn indices(self) -> (usize, usize, usize, usize) {
        match self {
            Self::Node(n) => (n, 0, 0, 0),
            Self::Session(n, s) => (n, s, 0, 0),
            Self::Workspace(n, s, w) => (n, s, w, 0),
            Self::Agent(n, s, w, a) => (n, s, w, a),
        }
    }
    pub fn seed(self) -> u32 {
        let (n, s, w, a) = self.indices();
        (n * 4096 + s * 512 + w * 32 + a) as u32
    }
}
pub fn noise(seed: u32) -> f32 {
    let mut x = seed.wrapping_add(0x9e3779b9);
    x = (x ^ (x >> 16)).wrapping_mul(0x85ebca6b);
    x = (x ^ (x >> 13)).wrapping_mul(0xc2b2ae35);
    ((x ^ (x >> 16)) & 0xffff) as f32 / 65536.0
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentState {
    Working,
    Blocked,
    Completed,
}
impl AgentState {
    pub fn color(self) -> [f32; 3] {
        match self {
            Self::Working => [0.05, 0.8, 1.0],
            Self::Blocked => [1.0, 0.32, 0.05],
            Self::Completed => [0.15, 0.95, 0.42],
        }
    }
    pub fn next(self) -> Self {
        match self {
            Self::Working => Self::Blocked,
            Self::Blocked => Self::Completed,
            Self::Completed => Self::Working,
        }
    }
}
pub fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[derive(Clone, Copy, Debug)]
pub struct Life {
    from: f32,
    target: f32,
    since: f32,
}
impl Life {
    pub fn alpha(self, time: f32) -> f32 {
        self.from + (self.target - self.from) * ease((time - self.since) / TRANSITION)
    }
    fn retarget(&mut self, target: f32, time: f32) {
        if self.target != target {
            self.from = self.alpha(time);
            self.target = target;
            self.since = time;
        }
    }
    pub fn entering(self) -> bool {
        self.target > 0.0
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Pulse {
    pub origin: Id,
    pub started: f32,
    pub color: [f32; 3],
}
#[derive(Clone, Debug)]
pub struct Event {
    pub origin: Id,
    pub started: f32,
    pub title: &'static str,
    pub text: String,
    pub color: [f32; 3],
}

pub struct Simulation {
    pub settings: Settings,
    pub entities: BTreeMap<Id, Life>,
    states: BTreeMap<Id, AgentState>,
    pub pulses: Vec<Pulse>,
    pub events: VecDeque<Event>,
    pub time: f32,
    pub running: bool,
    pub auto_activity: bool,
    pub period: f32,
    pub speed: f32,
    pub callouts: bool,
    pub controls: bool,
    next_beat: f32,
    sequence: usize,
}
impl Default for Simulation {
    fn default() -> Self {
        let mut sim = Self {
            settings: Settings {
                nodes: 0,
                ..Settings::default()
            },
            entities: BTreeMap::new(),
            states: BTreeMap::new(),
            pulses: vec![],
            events: VecDeque::new(),
            time: 0.0,
            running: true,
            auto_activity: true,
            period: 6.0,
            speed: 1.0,
            callouts: true,
            controls: true,
            next_beat: 2.0,
            sequence: 0,
        };
        sim.configure(Settings::default());
        sim
    }
}
impl Simulation {
    pub fn configure(&mut self, settings: Settings) {
        let settings = settings.bounded();
        if settings == self.settings {
            return;
        }
        let old = self.settings;
        let states_changed = settings.working != old.working || settings.blocked != old.blocked;
        self.settings = settings;
        for (id, life) in &mut self.entities {
            life.retarget(if settings.contains(*id) { 1.0 } else { 0.0 }, self.time);
        }
        for id in settings.ids() {
            self.entities.entry(id).or_insert(Life {
                from: 0.0,
                target: 1.0,
                since: self.time,
            });
        }
        self.states
            .retain(|id, _| settings.contains(*id) && !states_changed);
        self.pulses.retain(|p| settings.contains(p.origin));
        let origin = Id::Node(settings.nodes.saturating_sub(1));
        let (title, text, color) = if settings.nodes > old.nodes {
            (
                "MESH EXPANDING",
                format!("{} worker satellites joined", settings.nodes - old.nodes),
                [0.2, 0.9, 1.0],
            )
        } else if settings.nodes < old.nodes {
            (
                "MESH CONTRACTING",
                format!("{} worker satellites departing", old.nodes - settings.nodes),
                [1.0, 0.45, 0.15],
            )
        } else if states_changed {
            (
                "WORKLOAD REBALANCED",
                "Agent state distribution updated".into(),
                [0.1, 0.9, 0.6],
            )
        } else {
            (
                "TOPOLOGY UPDATED",
                "Session / workspace / agent constellation reshaping".into(),
                [0.65, 0.4, 1.0],
            )
        };
        // A departure callout remains anchored to a fading departing worker.
        let origin = if old.nodes > settings.nodes {
            Id::Node(settings.nodes)
        } else {
            origin
        };
        self.event(origin, title, text, color);
    }
    pub fn advance(&mut self, delta: f32) {
        if self.running {
            self.time += delta.clamp(0.0, 0.1) * self.speed.clamp(0.1, 3.0);
        }
        let time = self.time;
        self.entities
            .retain(|_, life| life.entering() || time - life.since < TRANSITION);
        self.pulses.retain(|p| time - p.started < PULSE_DURATION);
        self.events.retain(|e| time - e.started < 6.0);
        if self.auto_activity && self.running && time >= self.next_beat {
            self.trigger();
            self.next_beat = time + self.period.clamp(3.0, 15.0);
        }
    }
    pub fn state(&self, id: Id) -> AgentState {
        self.states.get(&id).copied().unwrap_or_else(|| {
            let value = noise(id.seed()) * 100.0;
            if value < self.settings.working as f32 {
                AgentState::Working
            } else if value < (self.settings.working + self.settings.blocked) as f32 {
                AgentState::Blocked
            } else {
                AgentState::Completed
            }
        })
    }
    pub fn counts(&self) -> [usize; 3] {
        let mut c = [0; 3];
        for id in self
            .entities
            .keys()
            .filter(|id| matches!(id, Id::Agent(..)) && self.settings.contains(**id))
        {
            c[match self.state(*id) {
                AgentState::Working => 0,
                AgentState::Blocked => 1,
                AgentState::Completed => 2,
            }] += 1;
        }
        c
    }
    pub fn trigger(&mut self) {
        if self.settings.nodes == 0 {
            return;
        }
        // Cycle workers; select a valid leaf when the topology has one.
        let n = self.sequence % self.settings.nodes;
        let s = self.sequence % self.settings.sessions.max(1);
        let w = (self.sequence / 2) % self.settings.workspaces.max(1);
        let a = (self.sequence / 3) % self.settings.agents.max(1);
        self.sequence = self.sequence.wrapping_add(1);
        let id = if self.settings.sessions == 0 {
            Id::Node(n)
        } else if self.settings.workspaces == 0 {
            Id::Session(n, s)
        } else if self.settings.agents == 0 {
            Id::Workspace(n, s, w)
        } else {
            Id::Agent(n, s, w, a)
        };
        let state = if matches!(id, Id::Agent(..)) {
            let state = self.state(id).next();
            self.states.insert(id, state);
            state
        } else {
            AgentState::Working
        };
        self.pulses.retain(|p| p.origin.indices().0 != n);
        self.pulses.push(Pulse {
            origin: id,
            started: self.time,
            color: state.color(),
        });
        let (title, text) = match state {
            AgentState::Working => (
                "WORK RESUMED",
                "Leaf activity propagating to the coordinator",
            ),
            AgentState::Blocked => ("ATTENTION REQUIRED", "An agent is awaiting input"),
            AgentState::Completed => ("TASK COMPLETED", "A unit of work has finished"),
        };
        self.event(id, title, text.into(), state.color());
    }
    fn event(&mut self, origin: Id, title: &'static str, text: String, color: [f32; 3]) {
        if self.events.len() >= 3 {
            self.events.pop_front();
        }
        self.events.push_back(Event {
            origin,
            started: self.time,
            title,
            text,
            color,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tick(sim: &mut Simulation, seconds: f32) {
        for _ in 0..(seconds * 20.0) as usize {
            sim.advance(0.05);
        }
    }
    #[test]
    fn counts_and_empty_topology() {
        let mut s = Simulation {
            auto_activity: false,
            ..Default::default()
        };
        assert_eq!(s.counts().iter().sum::<usize>(), 6 * 2 * 3 * 6);
        s.configure(Settings {
            nodes: 0,
            ..s.settings
        });
        tick(&mut s, 2.0);
        assert!(s.entities.is_empty());
        assert_eq!(s.counts(), [0; 3]);
        s.trigger();
        assert!(s.pulses.is_empty());
    }
    #[test]
    fn reversal_preserves_current_opacity_and_identity() {
        let mut s = Simulation {
            auto_activity: false,
            ..Default::default()
        };
        tick(&mut s, 2.0);
        let id = Id::Node(5);
        s.configure(Settings {
            nodes: 5,
            ..s.settings
        });
        tick(&mut s, 0.5);
        let alpha = s.entities[&id].alpha(s.time);
        s.configure(Settings {
            nodes: 6,
            ..s.settings
        });
        assert_eq!(s.entities[&id].alpha(s.time), alpha);
        tick(&mut s, 2.0);
        assert_eq!(s.entities[&id].alpha(s.time), 1.0);
    }
    #[test]
    fn removed_leaf_cancels_its_pulse() {
        let mut s = Simulation::default();
        s.trigger();
        assert_eq!(s.pulses.len(), 1);
        s.configure(Settings {
            agents: 0,
            ..s.settings
        });
        assert!(s.pulses.is_empty());
    }
    #[test]
    fn pause_freezes_lifecycles_and_events() {
        let mut s = Simulation::default();
        s.trigger();
        s.running = false;
        let t = s.time;
        tick(&mut s, 10.0);
        assert_eq!(s.time, t);
        assert_eq!(s.pulses.len(), 1);
    }
    #[test]
    fn controls_bound_work_and_dont_restart_unchanged_transitions() {
        let mut s = Simulation::default();
        let old = s.entities[&Id::Node(0)].since;
        s.configure(s.settings);
        assert_eq!(old, s.entities[&Id::Node(0)].since);
        s.configure(Settings {
            nodes: usize::MAX,
            sessions: usize::MAX,
            workspaces: usize::MAX,
            agents: usize::MAX,
            working: 150,
            blocked: 150,
        });
        assert_eq!(s.settings.bounded(), s.settings);
        assert_eq!(
            s.counts(),
            [MAX_NODES * MAX_SESSIONS * MAX_WORKSPACES * MAX_AGENTS, 0, 0]
        );
    }
}
