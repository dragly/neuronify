pub mod drawing;
pub mod gui;

use crate::settings::Setting;
use crate::stimuli::About;

pub const ABOUT: About = About {
    name: "Current source",
    description: "Feeds a steady current into the neurons it connects to.",
    size: None,
};

pub const CURRENT: Setting = Setting {
    label: "Current",
    description: "How strongly it pushes the neurons it is connected to.",
};
