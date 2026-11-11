# Possible changes to Neuronify's architecture

Changes under consideration, with what each would take and what is still open.
`current.md` describes the code as it is.
A change leaves this file once it is decided against, or done and described there.

## Untangle `update`

`Neuronify::update` mixes input timing, file loading, autosave, holds and their rings,
stepping, sound, landing previews, camera framing, the loupe and rendering
(see the list under "A frame" in `current.md`).
Not investigated yet.
