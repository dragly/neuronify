import QtQuick 2.0
import Neuronify 1.0

import ".."
import "../controls"
Neuron {
    id: neuronRoot

    objectName: "passiveNeuron"
    fileName: "neurons/PassiveNeuron.qml"
    imageSource: "qrc:/images/neurons/passive.png"
    inhibitoryImageSource: "qrc:/images/neurons/passive_inhibitory.png"

    engine: NeuronEngine {
        id: neuronEngine
        property real refractoryPeriod: 0.0e-3
        property real timeSinceFire: 99999.0
        fireOutput: 200.0e-6
        PassiveCurrent {
            id: passiveCurrent
        }
        onStepped: {
            if(timeSinceFire < refractoryPeriod) {
                neuronEngine.enabled = false
            } else {
                neuronEngine.enabled = true
            }
            timeSinceFire += dt
        }

        onFired: {
            timeSinceFire = 0.0
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

            BoundSlider {
                target: neuronEngine
                property: "refractoryPeriod"
                text: "Refractory period"
                unit: "ms"
                minimumValue: 0.0e-3
                maximumValue: 40e-3
                unitScale: 1e-3
                stepSize: 1e-3
                precision: 1
            }

            RestPotentialControl{
                engine: neuronEngine
            }
        }

    }

    savedProperties: PropertyGroup {
        property alias resistance: passiveCurrent.resistance
        property alias fireOutput: neuronEngine.fireOutput
        property alias refractoryPeriod: neuronEngine.refractoryPeriod
    }

}

