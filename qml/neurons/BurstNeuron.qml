import QtQuick 2.0
import Neuronify 1.0

import "qrc:/"
import "qrc:/qml/"
import "qrc:/qml/controls"

Neuron {
    id: neuronRoot

    objectName: "BurstNeuron"
    filename: "neurons/BurstNeuron.qml"
    imageSource: "qrc:/images/neurons/burst.png"
    inhibitoryImageSource: "qrc:/images/neurons/burst_inhibitory.png"

    engine: NeuronEngine {
        id: neuronEngine
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
        PropertiesPage {
            property string title: "Burst neuron"
            LabelControl {
                neuron: neuronRoot
            }

            SwitchControl{
                id: switchControl
                target: neuronRoot
                property: "inhibitory"
                checkedText: "Inhibitory"
                uncheckedText: "Excitatory"

            }
            spacing: 10
            RestPotentialControl{
                engine: neuronEngine
            }
        }
    }
}

