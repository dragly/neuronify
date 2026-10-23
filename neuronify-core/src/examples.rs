use hecs::World;

pub struct Example {
    pub name: &'static str,
    pub description: &'static str,
    pub nfy_contents: &'static str,
}

impl Example {
    pub fn spawn(&self) -> Result<World, String> {
        let simulation = crate::legacy::parse_legacy_nfy(self.nfy_contents)?;
        let mut world = World::new();
        crate::legacy::convert::spawn_legacy_simulation(&mut world, &simulation);
        Ok(world)
    }
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
                name: "Tutorial 1: Intro",
                description: "A single neuron driven by a current source, \
                    with a voltmeter showing its membrane potential.",
                nfy_contents: include_str!("../examples/tutorial_1_intro.nfy"),
            },
            Example {
                name: "Tutorial 2: Circuits",
                description: "Neuron A drives neuron B through a synapse. \
                    Connect B to C and see how C responds.",
                nfy_contents: include_str!("../examples/tutorial_2_circuits.nfy"),
            },
            Example {
                name: "Tutorial 3: Creation",
                description: "A ring of four neurons, ready for new items \
                    such as a touch sensor.",
                nfy_contents: include_str!("../examples/tutorial_3_creation.nfy"),
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
                nfy_contents: include_str!("../examples/leaky.nfy"),
            },
            Example {
                name: "Inhibitory",
                description: "An excitatory and an inhibitory neuron \
                    push the membrane potential of a third neuron up and down.",
                nfy_contents: include_str!("../examples/inhibitory.nfy"),
            },
            Example {
                name: "Adaptation",
                description: "An adaptive neuron gets harder to excite every time it fires, \
                    and recovers after a while.",
                nfy_contents: include_str!("../examples/adaptation.nfy"),
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
                nfy_contents: include_str!("../examples/input_summation.nfy"),
            },
            Example {
                name: "Prolonged Activity",
                description: "One touch sets off a loop of neurons \
                    that keeps firing until an adaptive neuron stops it.",
                nfy_contents: include_str!("../examples/prolonged_activity.nfy"),
            },
            Example {
                name: "Disinhibition",
                description: "Inhibiting an inhibitory neuron \
                    releases the brake on the output neuron.",
                nfy_contents: include_str!("../examples/disinhibition.nfy"),
            },
            Example {
                name: "Recurrent Inhibition",
                description: "An inhibitory neuron feeds back into the chain \
                    and lowers the output firing rate.",
                nfy_contents: include_str!("../examples/recurrent_inhibition.nfy"),
            },
            Example {
                name: "Reciprocal Inhibition",
                description: "Two pathways inhibit each other so that only one fires. \
                    Touch a sensor to switch sides.",
                nfy_contents: include_str!("../examples/reciprocal_inhibition.nfy"),
            },
            Example {
                name: "Lateral Inhibition",
                description: "The output neuron fires when the sensors are touched \
                    from right to left, but not from left to right.",
                nfy_contents: include_str!("../examples/lateral_inhibition.nfy"),
            },
            Example {
                name: "Lateral Inhibition 1",
                description: "The center neuron inhibits its neighbours, \
                    so the center dominates the output.",
                nfy_contents: include_str!("../examples/lateral_inhibition_1.nfy"),
            },
            Example {
                name: "Lateral Inhibition 2",
                description: "Every neuron inhibits its neighbours, \
                    which sharpens the difference between center and surround.",
                nfy_contents: include_str!("../examples/lateral_inhibition_2.nfy"),
            },
            Example {
                name: "Two Neuron Oscillator",
                description: "Two neurons wired to each other, \
                    each driven by a constant current.",
                nfy_contents: include_str!("../examples/two_neuron_oscillator.nfy"),
            },
            Example {
                name: "Rhythm Transformation",
                description: "Regular spike trains change frequency \
                    as they pass through synapses.",
                nfy_contents: include_str!("../examples/rythm_transformation.nfy"),
            },
            Example {
                name: "Types of Inhibition",
                description: "Presynaptic and postsynaptic inhibition side by side.",
                nfy_contents: include_str!("../examples/types_of_inhibition.nfy"),
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
                nfy_contents: include_str!("../examples/if_response.nfy"),
            },
            Example {
                name: "Refractory Period",
                description: "The refractory period caps the firing rate \
                    as the input current grows.",
                nfy_contents: include_str!("../examples/refractory_period.nfy"),
            },
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::LeakyNeuron;

    #[test]
    fn every_example_has_neurons() {
        for example in EXAMPLE_CATEGORIES.iter().flat_map(|c| c.examples) {
            let world = example.spawn().unwrap();
            let neurons = world.query::<&LeakyNeuron>().iter().count();
            assert!(neurons > 0, "{} has no neurons", example.name);
        }
    }
}
