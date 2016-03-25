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
//        AdaptationCurrent{}
        Current {
            property real boost: 0.0
            onFired: {
                if(boost < 0.1) {
                    boost = 4.0
                }
            }
            onStepped: {
                if(boost > 0.0) {
                    boost = boost - 1.0*dt
                } else {
                    boost = 0.0
                }
                current = -boost * (engine.voltage - 60.0)
            }
        }
    }
}

