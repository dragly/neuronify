import QtQuick 2.0
import Neuronify 1.0

import ".."

Neuron {
    objectName: "BurstNeuron"
    fileName: "neurons/BurstNeuron.qml"
    imageSource: "qrc:/images/neurons/burst.png"
    inhibitoryImageSource: "qrc:/images/neurons/burst_inhibitory.png"

    engine: NeuronEngine {
        fireOutput: 2.0
        PassiveCurrent {}
    }
}

