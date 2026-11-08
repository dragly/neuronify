use hecs::World;

pub enum Contents {
    Saved {
        file: &'static str,
        bytes: &'static [u8],
    },
}

macro_rules! saved {
    ($file:literal) => {
        Contents::Saved {
            file: $file,
            bytes: include_bytes!(concat!("../examples/", $file)),
        }
    };
}

pub struct Example {
    pub name: &'static str,
    pub description: &'static str,
    pub contents: Contents,
}

impl Example {
    pub fn file(&self) -> &'static str {
        match self.contents {
            Contents::Saved { file, .. } => file,
        }
    }

    pub fn spawn(&self) -> Result<World, String> {
        match self.contents {
            Contents::Saved { bytes, .. } => Ok(crate::serialization::load(bytes)?.world),
        }
    }
}

pub fn find(file: &str) -> Option<&'static Example> {
    EXAMPLE_CATEGORIES
        .iter()
        .flat_map(|category| category.examples)
        .find(|example| example.file() == file)
}

pub struct ExampleCategory {
    pub name: &'static str,
    pub examples: &'static [Example],
}

pub const EXAMPLE_CATEGORIES: &[ExampleCategory] = &[
    ExampleCategory {
        name: "Getting started",
        examples: &[
            Example {
                name: "1. Spikes",
                description: "Press the touch sensor and the neuron fires a spike.",
                contents: saved!("tutorial/1_spikes.neuronify"),
            },
            Example {
                name: "2. Leak currents",
                description: "The neuron builds up potential from incoming currents, \
                    but a continuous leak current brings it down again.",
                contents: saved!("tutorial/2_leaks.neuronify"),
            },
            Example {
                name: "3. Push and pull",
                description: "Excitatory neurons push others toward firing. \
                    Inhibitory neurons pull them back.",
                contents: saved!("tutorial/3_push_and_pull.neuronify"),
            },
            Example {
                name: "4. Choosing sides",
                description: "Two sides pull each other down, so only one of them fires.",
                contents: saved!("tutorial/4_choosing.neuronify"),
            },
            Example {
                name: "5. Sharpening edges",
                description: "Neighbors pull on each other and make an edge stand out.",
                contents: saved!("tutorial/5_sharpening_edges.neuronify"),
            },
            Example {
                name: "6. Keeping a rhythm",
                description: "Close a loop of neurons and it keeps a beat on its own.",
                contents: saved!("tutorial/6_keeping_a_rhythm.neuronify"),
            },
            Example {
                name: "7. Axons",
                description: "Spikes travel as action potentials along axons in real neurons.",
                contents: saved!("tutorial/7_axons.neuronify"),
            },
            Example {
                name: "8. Myelination",
                description: "Wrapping an axon in myelin makes spikes travel faster along it.",
                contents: saved!("tutorial/8_myelin.neuronify"),
            },
            Example {
                name: "9. Dendrites",
                description: "Axons can be connected to dendrites with synaptic connections.",
                contents: saved!("tutorial/9_dendrites.neuronify"),
            },
            Example {
                name: "10. Coincidence",
                description: "Weak inputs make a neuron fire only when they arrive together.",
                contents: saved!("tutorial/10_coincidence.neuronify"),
            },
            Example {
                name: "11. Inhibition on a branch",
                description: "Inhibition on a dendrite silences only the inputs beyond it.",
                contents: saved!("tutorial/11_inhibition_on_a_branch.neuronify"),
            },
        ],
    },
    ExampleCategory {
        name: "Neurons",
        examples: &[
            Example {
                name: "Adaptation",
                description: "This neuron gets harder to excite each time it fires, \
                    and recovers after a rest.",
                contents: saved!("adaptation.neuronify"),
            },
            Example {
                name: "Bursting",
                description: "One press makes this neuron fire a quick series of spikes.",
                contents: saved!("bursting.neuronify"),
            },
            Example {
                name: "Firing threshold",
                description: "A steady current makes a neuron fire only when it is strong enough, \
                    and faster the stronger it gets.",
                contents: saved!("firing_threshold.neuronify"),
            },
            Example {
                name: "Refractory period",
                description: "After each spike a neuron needs a short rest, \
                    which limits how fast it can fire.",
                contents: saved!("refractory_period.neuronify"),
            },
            Example {
                name: "Body temperature",
                description: "Neurons with channels from mammals at 37 °C, \
                    to compare with the textbook squid channels.",
                contents: saved!("body_temperature.neuronify"),
            },
        ],
    },
    ExampleCategory {
        name: "Circuits",
        examples: &[
            Example {
                name: "Summation",
                description: "One input is not enough to make the neuron fire, \
                    but several together, or one repeated quickly, are.",
                contents: saved!("summation.neuronify"),
            },
            Example {
                name: "Reverberation",
                description: "One press keeps a loop of neurons firing until one of them tires.",
                contents: saved!("reverberation.neuronify"),
            },
            Example {
                name: "Half-center oscillator",
                description: "Two neurons that hold each other back take turns firing, \
                    like the circuits behind walking.",
                contents: saved!("half_center_oscillator.neuronify"),
            },
            Example {
                name: "Feedback loops",
                description: "Feedback that excites makes a rhythm faster, \
                    and feedback that inhibits makes it slower.",
                contents: saved!("feedback_loops.neuronify"),
            },
        ],
    },
    ExampleCategory {
        name: "Inhibition",
        examples: &[
            Example {
                name: "Disinhibition",
                description: "Holding back the neuron that holds back the output \
                    lets the output fire.",
                contents: saved!("disinhibition.neuronify"),
            },
            Example {
                name: "Feedback inhibition",
                description: "The output wakes an inhibitory neuron \
                    that reaches back and slows the chain down.",
                contents: saved!("feedback_inhibition.neuronify"),
            },
            Example {
                name: "Lateral inhibition",
                description: "Neighbors hold each other back, \
                    so the strongest touch stands out clearly.",
                contents: saved!("lateral_inhibition.neuronify"),
            },
            Example {
                name: "Adaptive inhibition",
                description: "An inhibitory neuron adapts, \
                    and the neuron it holds back fires more and more.",
                contents: saved!("adaptive_inhibition.neuronify"),
            },
        ],
    },
    ExampleCategory {
        name: "Senses",
        examples: &[
            Example {
                name: "Receptive fields",
                description: "Each neuron watches its own spot of the picture \
                    and fires when a moving dot crosses it.",
                contents: saved!("receptive_fields.neuronify"),
            },
            Example {
                name: "Listening to a neuron",
                description: "Hear a neuron click as stripes of light pass \
                    at the angle it prefers.",
                contents: saved!("listening_to_a_neuron.neuronify"),
            },
            Example {
                name: "Direction selectivity",
                description: "The output fires when you press the sensors from right to left, \
                    but not from left to right.",
                contents: saved!("direction_selectivity.neuronify"),
            },
        ],
    },
    ExampleCategory {
        name: "Memory",
        examples: &[
            Example {
                name: "Ring attractor",
                description: "Press a touch sensor and a bump of activity forms on the ring \
                    and stays after you let go. Press another to move it.",
                contents: saved!("labs/ring_attractor.neuronify"),
            },
            Example {
                name: "Ring attractor with noise",
                description: "Random input keeps every cell flickering, \
                    but a bump you start still holds its place.",
                contents: saved!("labs/ring_attractor_noise.neuronify"),
            },
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::LeakyNeuron;
    use hecs::Entity;

    #[test]
    fn examples_are_found_by_their_file() {
        assert_eq!(find("bursting.neuronify").unwrap().name, "Bursting");
        assert!(find("missing.nfy").is_none());
    }

    #[test]
    fn every_bundled_example_file_is_listed() {
        let listed: Vec<&str> = EXAMPLE_CATEGORIES
            .iter()
            .flat_map(|category| category.examples)
            .map(Example::file)
            .collect();
        let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/examples"));
        let mut folders = vec![root.to_path_buf()];
        let mut bundled = Vec::new();
        while let Some(folder) = folders.pop() {
            for entry in std::fs::read_dir(folder).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    folders.push(path);
                } else if path
                    .extension()
                    .is_some_and(|extension| extension == "neuronify")
                {
                    let file = path.strip_prefix(root).unwrap().to_string_lossy();
                    bundled.push(file.replace('\\', "/"));
                }
            }
        }
        assert!(bundled.iter().any(|file| file.starts_with("tutorial/")));
        assert!(bundled.iter().any(|file| file.starts_with("labs/")));
        for file in bundled {
            assert!(listed.contains(&file.as_str()), "{file} is not listed");
        }
    }

    #[test]
    fn every_example_has_neurons() {
        for example in EXAMPLE_CATEGORIES.iter().flat_map(|c| c.examples) {
            let world = example.spawn().unwrap();
            let neurons = world.query::<(Entity, &LeakyNeuron)>().iter().count();
            assert!(neurons > 0, "{} has no neurons", example.name);
        }
    }
}
