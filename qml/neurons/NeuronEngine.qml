import QtQuick 2.0

import Neuronify 1.0

NeuronEngineBase {
    id: engine
    property real fakeFireOutput: 0.0
    savedProperties: PropertyGroup {
        property alias initialPotential: engine.initialPotential
        property alias restingPotential: engine.restingPotential
        property alias threshold: engine.threshold
        property alias voltage: engine.voltage
        property alias capacitance: engine.capacitance
        property alias fireOutput: engine.fakeFireOutput
    }
}
