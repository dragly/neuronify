import QtQuick 2.0
import Neuronify 1.0

import ".."

Neuron {
    objectName: "adaptationNeuron"
    fileName: "neurons/AdaptationNeuron.qml"

    color: fireOutput > 0.0 ? "green" : "yellow"
    engine: NeuronEngine {
        fireOutput: 2.0
        PassiveCurrent {
        }
    }
}

