import QtQuick 2.0
import QtQuick.Controls 1.0

import Neuronify 1.0


import "qrc:/"
import "qrc:/qml/"
import "qrc:/qml/controls"


Neuron {
    id: neuronRoot


    objectName: "adaptationNeuron"
    fileName: "neurons/AdaptationNeuron.qml"
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
    }


    controls: Component {
        PropertiesPage {
            property string title: "Adaptive neuron"
            LabelControl {
                neuron: neuronRoot
            }
            SynapticOutputControl {
                engine: neuronEngine
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


    savedProperties: PropertyGroup {
        property alias label: neuronRoot.label
        property alias fireOutput: neuronEngine.fireOutput
        property alias adaptation: adaptationCurrent.adaptation
        property alias timeConstant: adaptationCurrent.timeConstant


        // Do we need to save these?
        property alias resistance: passiveCurrent.resistance
        property alias capacitance: neuronEngine.capacitance
        property alias restingPotential: neuronEngine.restingPotential
        property alias initialPotential: neuronEngine.initialPotential
        property alias threshold: neuronEngine.threshold
        property alias synapticTimeConstant: neuronEngine.synapticTimeConstant
        property alias synapticPotential: neuronEngine.synapticPotential
    }
}

