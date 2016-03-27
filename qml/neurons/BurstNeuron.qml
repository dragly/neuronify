import QtQuick 2.0
import Neuronify 1.0

import ".."

Neuron {
    property alias fireOutput: engine.fireOutput

    objectName: "BurstNeuron"
    fileName: "neurons/BurstNeuron.qml"
    imageSource: "qrc:/images/neurons/burst.png"
    inhibitoryImageSource: "qrc:/images/neurons/burst_inhibitory.png"

    engine: NeuronEngine {
        id: engine
        fireOutput: 2.0
        PassiveCurrent {}
        Current {
            property real boost: 0.0
            onFired: {
                if(boost < 1.0e-9) {
                    boost = 100.0e-6
                }
            }
            onStepped: {
                if(boost > 0.0) {
                    boost = boost - 1000.0e-6*dt
                } else {
                    boost = 0.0
                }
                current = -boost * (engine.voltage - 60.0e-3)
            }
        }
    }
}

