// Re-export canonical component types under their old Classic* names
// for backwards compatibility within the legacy module.
pub use crate::components::{
    AdaptationCurrent as ClassicAdaptationCurrent, Annotation as ClassicAnnotation,
    CurrentClamp as ClassicCurrentClamp, CurrentSynapse as ClassicCurrentSynapse,
    ImmediateFireSynapse as ClassicImmediateFireSynapse, Inhibitory as ClassicInhibitory,
    LIFDynamics as ClassicNeuronDynamics, LIFNeuron as ClassicNeuron,
    LeakCurrent as ClassicLeakCurrent, TouchSensor as ClassicTouchSensor,
    VoltmeterSize as ClassicVoltmeterSize,
};
