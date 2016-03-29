import QtQuick 2.0
import Neuronify 1.0

import ".."
import "../controls"

Neuron {
    id: neuronRoot
    property alias fireOutput: neuronEngine.fireOutput
    property alias resistance: passiveCurrent.resistance

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
        NeuronControls {
            neuron: neuronRoot
            engine: neuronEngine

            BoundSlider {
                target: passiveCurrent
                property: "resistance"
                minimumValue: 1e3
                maximumValue: 100e3
                unitScale: 1e3
                stepSize: 1e3
                precision: 1
                text: "Membrane resistance"
                unit: "kΩ"
            }
            RestPotentialControl{
                engine: neuronEngine
            }
        }
    }

    savedProperties: PropertyGroup {
        property alias resistance: neuronRoot.resistance
    }
}

