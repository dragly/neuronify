pub mod drawing;
pub mod gui;

use crate::stimuli::About;

pub const ABOUT: About = About {
    name: "Current source",
    description: "Feeds a steady current into the neurons it connects to.",
    size: None,
};
