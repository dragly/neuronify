pub mod drawing;
pub mod gui;

use serde::{Deserialize, Serialize};

use crate::stimuli::About;

pub const ABOUT: About = About {
    name: "Touch sensor",
    description: "Fires a spike each time you press it.",
    size: None,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TouchSensor;
