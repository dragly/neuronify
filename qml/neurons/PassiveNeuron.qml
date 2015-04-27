import QtQuick 2.0
import Neuronify 1.0

import ".."

Neuron {
    property alias fireOutput: nEngine.fireOutput
    objectName: "adaptationNeuron"
    fileName: "neurons/PassiveNeuron.qml"
    imageSource: "qrc:/images/neurons/passive.png"
    inhibitoryImageSource: "qrc:/images/neurons/passive_inhibitory.png"

    engine: NeuronEngine {
        id: nEngine
        fireOutput: 2.0
        PassiveCurrent {}
    }

}

