import QtQuick 2.0
import Neuronify 1.0

import ".."
import "../controls"
Neuron {
    id: neuronRoot
    property alias resistance: passiveCurrent.resistance
    property alias fireOutput: neuronEngine.fireOutput

    objectName: "passiveNeuron"
    fileName: "neurons/PassiveNeuron.qml"
    imageSource: "qrc:/images/neurons/passive.png"
    inhibitoryImageSource: "qrc:/images/neurons/passive_inhibitory.png"

    engine: NeuronEngine {
        id: neuronEngine
        fireOutput: 200.0e-6
        PassiveCurrent {
            id: passiveCurrent
        }
    }

    controls: Component {
        NeuronControls {
            neuron: neuronRoot
            engine: neuronEngine
            Text {
                text: "Passive properties:"
            }
            BoundSlider {
                target: passiveCurrent
                property: "resistance"
                minimumValue: 0.1e3
                maximumValue: 100e3
                unitScale: 1e3
                stepSize: 1e2
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
        property alias fireOutput: neuronRoot.fireOutput
    }

}

