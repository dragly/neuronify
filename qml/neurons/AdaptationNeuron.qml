import QtQuick 2.0
import QtQuick.Controls 1.0

import Neuronify 1.0

import ".."

Neuron {
    property alias adaptation: adaptationCurrent.adaptation

    objectName: "adaptationNeuron"
    fileName: "neurons/AdaptationNeuron.qml"
    imageSource: "qrc:/images/neurons/adaptive.png"
    inhibitoryImageSource: "qrc:/images/neurons/adaptive_inhibitory.png"

    engine: NeuronEngine {
        id: engine
        fireOutput: 2.0
        PassiveCurrent {
        }
        AdaptationCurrent {
            id: adaptationCurrent
            adaptation: 10.0
        }
    }

    controls: Component {
        Column {
            Text {
                text: "Adaptation: " + adaptation.toFixed(1)
            }
            Slider {
                id: adaptationSlider
                width: parent.width
                minimumValue: 0.0
                maximumValue: 20.0
                value: adaptation
            }
            Binding {
                target: adaptationCurrent
                property: "adaptation"
                value: adaptationSlider.value
            }
        }
    }

    Component.onCompleted: {
        dumpableProperties.concat("adaptation")
    }
}

