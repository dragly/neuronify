import QtQuick 2.0
import Neuronify 1.0

import ".."

Neuron {
    objectName: "adaptationNeuron"
    fileName: "neurons/AdaptationNeuron.qml"
    imageSource: "qrc:/images/creators/neurons/adaptive.png"
    inhibitoryImageSource: "qrc:/images/creators/neurons/adaptive_inhibitory.png"

    engine: NeuronEngine {
        fireOutput: 2.0
        PassiveCurrent {

        }
        AdaptationCurrent {
            adaptation: 10.0
        }
    }
}

