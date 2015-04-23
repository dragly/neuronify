import QtQuick 2.0
import Neuronify 1.0

import ".."

Neuron {
    objectName: "adaptationNeuron"
    fileName: "neurons/PassiveNeuron.qml"
    imageSource: "qrc:/images/creators/neurons/passive.png"
    inhibitoryImageSource: "qrc:/images/creators/neurons/passive_inhibitory.png"

    engine: NeuronEngine {
        fireOutputMagnitude: 2.0
        PassiveCurrent {}
    }
}

