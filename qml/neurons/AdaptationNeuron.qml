import QtQuick 2.0
import QtQuick.Controls 1.0

import Neuronify 1.0

import ".."
import "../controls"

Neuron {
    id: neuronRoot
    property alias adaptation: adaptationCurrent.adaptation
    property alias timeConstant: adaptationCurrent.timeConstant
    property alias fireOutput: neuronEngine.fireOutput

    objectName: "adaptationNeuron"
    fileName: "neurons/AdaptationNeuron.qml"
    imageSource: "qrc:/images/neurons/adaptive.png"
    inhibitoryImageSource: "qrc:/images/neurons/adaptive_inhibitory.png"

    engine: NeuronEngine {
        id: neuronEngine
        fireOutput: 200.0e-6
        PassiveCurrent {}
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
            Text {
                text: "Adaptation: " + adaptation.toFixed(1)
            }
            BoundSlider {
                width: parent.width
                minimumValue: 0.0
                maximumValue: 20.0
                target: adaptationCurrent
                property: "adaptation"
            }
            Text {
                text: "Time constant: " + timeConstant.toFixed(1)
            }
            BoundSlider {
                width: parent.width
                minimumValue: 0.0
                maximumValue: 10.0
                target: adaptationCurrent
                property: "timeConstant"
            }
        }
    }

    Component.onCompleted: {
        dumpableProperties = dumpableProperties.concat("adaptation")
    }
}

