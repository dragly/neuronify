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
            BoundSlider {
                target: passiveCurrent
                property: "resistance"
                minimumValue: 1
                maximumValue: 1000
                unitScale: 1
                stepSize: 1
                precision: 1
                text: "Membrane resistance"
                unit: "Ω"
            }
        }
    }

    Component.onCompleted: {
        dumpableProperties = dumpableProperties.concat("resistance")
    }

}

