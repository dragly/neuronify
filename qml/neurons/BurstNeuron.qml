import QtQuick 2.0
import Neuronify 1.0

import "qrc:/"
import "qrc:/qml/"
import "qrc:/qml/controls"

Neuron {
    id: neuronRoot

    objectName: "BurstNeuron"
    fileName: "neurons/BurstNeuron.qml"
    imageSource: "qrc:/images/neurons/burst.png"
    inhibitoryImageSource: "qrc:/images/neurons/burst_inhibitory.png"

    engine: NeuronEngine {
        id: neuronEngine
        fireOutput: 200.0e-6
        PassiveCurrent {
            id: passiveCurrent
        }
        Current {
            property real boost: 0.0
            onFired: {
                if(boost < 1.0e-9) {
                    boost = 200.0e-6
                }
            }
            onStepped: {
                if(boost > 0.0) {
                    boost = boost - 1000.0e-6*dt
                } else {
                    boost = 0.0
                }
                current = -boost * (neuronEngine.voltage - 60.0e-3)
            }
        }
    }

    controls: Component {
        Column {
            LabelControl {
                neuron: neuronRoot
            }
            SynapticOutputControl {
                engine: neuronEngine
            }
            RestPotentialControl{
                engine: neuronEngine
            }
        }
    }

    savedProperties: PropertyGroup {
        property alias label: neuronRoot.label
    }
}

