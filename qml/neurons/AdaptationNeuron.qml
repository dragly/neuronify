import QtQuick 2.0
import QtQuick.Controls 1.0

import Neuronify 1.0

import ".."
import "../controls"

Neuron {
    id: neuronRoot
    property alias adaptation: adaptationCurrent.adaptation
    property alias timeConstant: adaptationCurrent.timeConstant
    property alias resistance: passiveCurrent.resistance
    property alias fireOutput: neuronEngine.fireOutput

    objectName: "adaptationNeuron"
    fileName: "neurons/AdaptationNeuron.qml"
    imageSource: "qrc:/images/neurons/adaptive.png"
    inhibitoryImageSource: "qrc:/images/neurons/adaptive_inhibitory.png"

    engine: NeuronEngine {
        id: neuronEngine
        fireOutput: 200.0e-6
        PassiveCurrent {
            id: passiveCurrent
        }
        AdaptationCurrent {
            id: adaptationCurrent
            adaptation: 10.0e-6
            timeConstant: 500.0e-3
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

            BoundSlider {
                width: parent.width
                minimumValue: 0.0
                maximumValue: 100e-6
                target: adaptationCurrent
                property: "adaptation"
                text: "Adaptation"
                unitScale: 1e-6
                stepSize: 1e-7
                unit: "uS"
                precision: 1
            }
            BoundSlider {
                width: parent.width
                minimumValue: 0.0
                maximumValue: 50.0e-3
                target: adaptationCurrent
                property: "timeConstant"
                text: "Time constant"
                unit: "ms"
                unitScale: 1e-3
                stepSize: 1e-4
                precision: 1
            }
            RestPotentialControl{
                engine: neuronEngine
            }
        }
    }

    Component.onCompleted: {
        dumpableProperties = dumpableProperties.concat(
                    ["resistance",
                    "adaptation",
                    "timeConstant"])
    }
}

