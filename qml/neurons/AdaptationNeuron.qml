import QtQuick 2.0
import QtQuick.Controls 1.0

import Neuronify 1.0


import "qrc:/"
import "qrc:/qml/"
import "qrc:/qml/controls"


Neuron {
    id: neuronRoot


    objectName: "adaptationNeuron"
    filename: "neurons/AdaptationNeuron.qml"
    imageSource: "qrc:/images/neurons/adaptive.png"
    inhibitoryImageSource: "qrc:/images/neurons/adaptive_inhibitory.png"

    engine: NeuronEngine {
        id: neuronEngine
        PassiveCurrent {
            id: passiveCurrent
        }
        AdaptationCurrent {
            id: adaptationCurrent
            adaptation: 50.0e-6
            timeConstant: 300.0e-3
        }

        savedProperties: PropertyGroup {
            property alias adaptation: adaptationCurrent.adaptation
            property alias timeConstant: adaptationCurrent.timeConstant
        }
    }

    controls: Component {
        PropertiesPage {
            property string title: "Adaptive neuron"
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
            AdaptationControl{
                current: adaptationCurrent
            }
            AdaptationTimeConstantControl{
                current: adaptationCurrent
            }
            spacing: 10
            RestPotentialControl{
                engine: neuronEngine
            }
        }
    }
}

