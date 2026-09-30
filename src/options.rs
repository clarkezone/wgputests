use std::time::Duration;

use crate::ui::Experience;

pub const HELP: &str = "Usage: native-3d-app [--screensaver] [--scene cube|logic-core|orbital-sphere|prismatic] [--rotate-seconds N|--no-rotate]\n\nWith no arguments, the interactive gallery opens as usual. Screensaver mode fills the display, hides the interface, rotates scenes every three minutes, and exits on input.\n";

#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub screensaver: bool,
    pub scene: Experience,
    pub rotation: Option<Duration>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            screensaver: false,
            scene: Experience::LogicCore,
            rotation: Some(Duration::from_secs(180)),
        }
    }
}

impl Options {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut options = Self::default();
        for arg in args {
            match arg.as_str() {
                "--screensaver" | "--screensaver=org.omarchy.screensaver" => {
                    options.screensaver = true;
                }
                "--no-rotate" => options.rotation = None,
                _ if arg.starts_with("--scene=") => {
                    let name = &arg[8..];
                    options.scene = match name {
                        "cube" => Experience::Cube,
                        "logic-core" => Experience::LogicCore,
                        "orbital-sphere" => Experience::OrbitalSphere,
                        "prismatic" => Experience::Prismatic,
                        _ => return Err(format!("unknown scene: {name}")),
                    };
                }
                _ if arg.starts_with("--rotate-seconds=") => {
                    let value = &arg[17..];
                    let seconds = value
                        .parse::<u64>()
                        .map_err(|_| format!("invalid rotation interval: {value}"))?;
                    if seconds == 0 {
                        return Err("rotation interval must be positive".into());
                    }
                    options.rotation = Some(Duration::from_secs(seconds));
                }
                _ => return Err(format!("unknown argument: {arg}")),
            }
        }
        Ok(options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_keeps_interactive_gallery() {
        let options = Options::parse([]).unwrap();
        assert!(!options.screensaver);
        assert_eq!(options.scene, Experience::LogicCore);
    }

    #[test]
    fn screensaver_scene_and_interval_are_configurable() {
        let options = Options::parse([
            "--screensaver".into(),
            "--scene=prismatic".into(),
            "--rotate-seconds=60".into(),
        ])
        .unwrap();
        assert!(options.screensaver);
        assert_eq!(options.scene, Experience::Prismatic);
        assert_eq!(options.rotation, Some(Duration::from_secs(60)));
    }
}
