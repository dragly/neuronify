use hecs::World;

pub enum Contents {
    Legacy {
        file: &'static str,
        json: &'static str,
    },
    Saved {
        file: &'static str,
        bytes: &'static [u8],
    },
}

macro_rules! legacy {
    ($file:literal) => {
        Contents::Legacy {
            file: $file,
            json: include_str!(concat!("../examples/", $file)),
        }
    };
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
            Contents::Legacy { file, .. } | Contents::Saved { file, .. } => file,
        }
    }

    pub fn spawn(&self) -> Result<World, String> {
        match self.contents {
            Contents::Legacy { json, .. } => {
                let simulation = crate::legacy::parse_legacy_nfy(json)?;
                let mut world = World::new();
                crate::legacy::convert::spawn_legacy_simulation(&mut world, &simulation);
                Ok(world)
            }
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
                name: "8. Dendrites",
                description: "Axons can be connected to dendrites with synaptic connections.",
                contents: saved!("tutorial/8_dendrites.neuronify"),
            },
            Example {
                name: "9. Coincidence",
                description: "Weak inputs make a neuron fire only when they arrive together.",
                contents: saved!("tutorial/9_coincidence.neuronify"),
            },
        ],
    },
    ExampleCategory {
        name: "Neurons",
        examples: &[
            Example {
                name: "Leaky",
                description: "A leaky integrate-and-fire neuron relaxes back to rest \
                    unless something keeps driving it.",
                contents: legacy!("leaky.nfy"),
            },
            Example {
                name: "Inhibitory",
                description: "An excitatory and an inhibitory neuron \
                    push the membrane potential of a third neuron up and down.",
                contents: legacy!("inhibitory.nfy"),
            },
            Example {
                name: "Adaptation",
                description: "An adaptive neuron gets harder to excite every time it fires, \
                    and recovers after a while.",
                contents: legacy!("adaptation.nfy"),
            },
            Example {
                name: "Burst",
                description: "A burst neuron keeps firing for a while \
                    after a single touch.",
                contents: legacy!("burst.nfy"),
            },
        ],
    },
    ExampleCategory {
        name: "Circuits",
        examples: &[
            Example {
                name: "Input Summation",
                description: "Inputs add up across neurons and over time \
                    before the output neuron reaches threshold.",
                contents: legacy!("input_summation.nfy"),
            },
            Example {
                name: "Prolonged Activity",
                description: "One touch sets off a loop of neurons \
                    that keeps firing until an adaptive neuron stops it.",
                contents: legacy!("prolonged_activity.nfy"),
            },
            Example {
                name: "Disinhibition",
                description: "Inhibiting an inhibitory neuron \
                    releases the brake on the output neuron.",
                contents: legacy!("disinhibition.nfy"),
            },
            Example {
                name: "Recurrent Inhibition",
                description: "An inhibitory neuron feeds back into the chain \
                    and lowers the output firing rate.",
                contents: legacy!("recurrent_inhibition.nfy"),
            },
            Example {
                name: "Reciprocal Inhibition",
                description: "Two pathways inhibit each other so that only one fires. \
                    Touch a sensor to switch sides.",
                contents: legacy!("reciprocal_inhibition.nfy"),
            },
            Example {
                name: "Lateral Inhibition",
                description: "The output neuron fires when the sensors are touched \
                    from right to left, but not from left to right.",
                contents: legacy!("lateral_inhibition.nfy"),
            },
            Example {
                name: "Lateral Inhibition 1",
                description: "The center neuron inhibits its neighbors, \
                    so the center dominates the output.",
                contents: legacy!("lateral_inhibition_1.nfy"),
            },
            Example {
                name: "Lateral Inhibition 2",
                description: "Every neuron inhibits its neighbors, \
                    which sharpens the difference between center and surround.",
                contents: legacy!("lateral_inhibition_2.nfy"),
            },
            Example {
                name: "Two Neuron Oscillator",
                description: "Two neurons wired to each other, \
                    each driven by a constant current.",
                contents: legacy!("two_neuron_oscillator.nfy"),
            },
            Example {
                name: "Rhythm Transformation",
                description: "Regular spike trains change frequency \
                    as they pass through synapses.",
                contents: legacy!("rythm_transformation.nfy"),
            },
            Example {
                name: "Types of Inhibition",
                description: "Presynaptic and postsynaptic inhibition side by side.",
                contents: legacy!("types_of_inhibition.nfy"),
            },
        ],
    },
    ExampleCategory {
        name: "Inputs and Meters",
        examples: &[
            Example {
                name: "Generators",
                description: "A steady current, an alternating current, regular spikes \
                    and random spikes each drive a neuron, \
                    and a spike detector shows when each one fires.",
                contents: legacy!("generators.nfy"),
            },
            Example {
                name: "Firing Rate Plot",
                description: "A spike generator drives a neuron, \
                    and a plot shows how often the neuron fires.",
                contents: legacy!("frPlot.nfy"),
            },
            Example {
                name: "Visual Input",
                description: "Two visual inputs look at a moving bar \
                    through different receptive fields and drive one neuron.",
                contents: legacy!("visualInput.nfy"),
            },
        ],
    },
    ExampleCategory {
        name: "Textbook",
        examples: &[
            Example {
                name: "IF Response",
                description: "An integrate-and-fire neuron fires faster with more current, \
                    once the current is enough to reach threshold.",
                contents: legacy!("if_response.nfy"),
            },
            Example {
                name: "Refractory Period",
                description: "The refractory period caps the firing rate \
                    as the input current grows.",
                contents: legacy!("refractory_period.nfy"),
            },
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::LeakyNeuron;

    #[test]
    fn examples_are_found_by_their_file() {
        assert_eq!(find("leaky.nfy").unwrap().name, "Leaky");
        assert!(find("missing.nfy").is_none());
    }

    #[test]
    fn every_bundled_neuronify_1_file_is_listed() {
        let listed: Vec<&str> = EXAMPLE_CATEGORIES
            .iter()
            .flat_map(|category| category.examples)
            .map(Example::file)
            .collect();
        let folder = concat!(env!("CARGO_MANIFEST_DIR"), "/examples");
        for entry in std::fs::read_dir(folder).unwrap() {
            let file = entry.unwrap().file_name().to_string_lossy().into_owned();
            let example = file.ends_with(".nfy");
            let replaced_by_the_tutorial = file.starts_with("tutorial_");
            if example && file != "empty.nfy" && !replaced_by_the_tutorial {
                assert!(listed.contains(&file.as_str()), "{file} is not listed");
            }
        }
    }

    #[test]
    fn every_example_has_neurons() {
        for example in EXAMPLE_CATEGORIES.iter().flat_map(|c| c.examples) {
            let world = example.spawn().unwrap();
            let neurons = world.query::<&LeakyNeuron>().iter().count();
            assert!(neurons > 0, "{} has no neurons", example.name);
        }
    }
}
