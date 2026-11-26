# Possible changes to Neuronify's architecture

Changes under consideration, with what each would take and what is still open.
`current.md` describes the code as it is.
A change leaves this file once it is decided against, or done and described there.

## Give each element kind one home

Most element kinds are a set of components that each layer recognizes on its own,
by checking which components an entity has.
So a new kind touches many files, and several of them hold lists the compiler doesn't check:
`copy_edited` for undo, `Fingerprint::of`, the inspector's checks, `bounds` and the spawn functions.
Adding the spike detector touched 35 files of app code, the loudspeaker 23,
and letting one visual input feed several receptive fields 40.

The meters are done (see "Where the code is" in `current.md`).
Before, a meter's name appeared in 13 to 18 files outside `measurement`.
Now it appears in 3 to 5, outside tests:
the undo list in `editor/history`, `legacy::convert` and the current save version for each of them,
`components` and `simulation::engine` for the voltmeter,
and `app` and `audio` for the sound the loudspeaker makes.
A new meter needs its file and folder, a variant in `Meter` and `Meter::ALL`,
and an arm in each of the 16 matches in `measurement.rs`, `measurement/gui.rs` and `measurement/drawing.rs`.

Still outside the meters' folders, and staying there,
since they follow formats from outside (Neuronify 1 files and older saves) or the engine's layout:

- what Neuronify 1 files become, in `legacy::convert`,
  which starts each meter from `Meter::builder` and wires it through `Meter::of`,
- the voltmeter probes and `Fingerprint::of` in `simulation::engine`,
- the save schema, which still takes a new version for a new meter.

The stimuli are done the same way (`Element::Stimulus` wraps a `Stimulus` for the rail and the editor).
A stimulus still appears outside `stimuli` in the undo list, `legacy::convert`, the current save version
and `simulation::engine`, for the same reasons as a meter,
in `editor` for what pressing a touch sensor and wiring a picture to a field do,
and in `simulation::stimulation` for touch sensors.
Notes are done as well, in `note` without an enum, since there is one kind.
A note still appears outside it in the undo list, `legacy::convert` and the current save version,
in `editor` and `editor::targets` for resizing and picking it,
in `rendering::bounds`, `rendering::labels` and `rendering::selection`, which use its size as they use `MeterSize`,
and in `app`, which opens the example a link points to.
Neurons would come next.
A neuron's kind is a combination (`LeakyNeuron` plus maybe `AdaptationCurrent`,
`BurstCurrent` or an inhibitory `NeuronType`),
so it would help to store the kind, which takes a save version.

Open questions:

- whether bounds belong to the model instead of `serialization`,
  since the meters, stimuli and notes now use `within` and its neighbours from there.

## Untangle `update`

`Neuronify::update` mixes input timing, file loading, autosave, holds and their rings,
stepping, sound, landing previews, camera framing, the loupe and rendering
(see the list under "A frame" in `current.md`).
Not investigated yet.
