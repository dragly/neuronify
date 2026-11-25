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
Now the voltmeter appears in 3 (`components`, `legacy::convert`, `simulation::engine`),
the spike detector and the rate plot in 2 or 3,
and the loudspeaker in 9, four of them the sound it makes in `app` and `audio`.
A new meter needs its file and folder, a variant in `Meter` and `Meter::ALL`,
and an arm in each of the 13 matches in `measurement.rs`, `measurement/gui.rs` and `measurement/drawing.rs`.
The compiler points at each match, but not at a meter left out of `Meter::ALL`.

Still outside the meters' folders:

- the names and descriptions of their settings, in the one `Setting` table in `settings.rs`,
- wire rows and labels in `rendering::connections` and `rendering::labels`,
  and the loudspeaker's pad radius in `rendering::selection`,
- what Neuronify 1 files become, in `legacy::convert`,
- the voltmeter probes and `Fingerprint::of` in `simulation::engine`,
- the save schema, which still takes a new version for a new meter.

The inputs would come next, since `Element` already lists them
(current sources, generators, the touch sensor, the receptive field and the picture),
then notes, then neurons.
A neuron's kind is a combination (`LeakyNeuron` plus maybe `AdaptationCurrent`,
`BurstCurrent` or an inhibitory `NeuronType`),
so it would help to store the kind, which takes a save version.

Open questions:

- whether bounds belong to the model instead of `serialization`,
  since the meters now use `within` from there,
- whether the `Setting` texts should move into the kinds
  or stay in one table, where all the wording can be read in one place.

## Untangle `update`

`Neuronify::update` mixes input timing, file loading, autosave, holds and their rings,
stepping, sound, landing previews, camera framing, the loupe and rendering
(see the list under "A frame" in `current.md`).
Not investigated yet.
