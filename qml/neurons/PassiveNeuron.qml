import QtQuick 2.6
import QtQuick.Controls 1.4

import Neuronify 1.0

import ".."
import "../controls"
import "../style"

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
        fireOutput: 300.0e-6
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
        PropertiesPage {
            property string title: "Passive neuron"
            property StackView stackView: Stack.view
            spacing: 0
            PropertiesItem {
                text: "Label"
                info: neuronRoot.label
                LabelControl {
                    neuron: neuronRoot
                }
            }
            PropertiesItem {
                text: "Potentials"
                info: "Vr: " + (neuronEngine.restingPotential * 1e3).toFixed(1) + " mV, " +
                      "Vi: " + (neuronEngine.initialPotential * 1e3).toFixed(1) + " mV, " +
                      "Vt: " + (neuronEngine.threshold * 1e3).toFixed(1) + " mV "
                RestingPotentialControl{
                    id: restingPotentialControl
                    engine: neuronEngine
                }

                InitialPotentialControl{
                    engine: neuronEngine
                }

                ThresholdControl{
                    engine: neuronEngine
                }
            }
            PropertiesItem {
                text: "Membrane"
                info: "C: " + (neuronEngine.capacitance * 1e9).toFixed(1) + " nF, " +
                      "R: " + (passiveCurrent.resistance * 1e-3).toFixed(1) + " kΩ, "
                CapacitanceControl{
                    engine: neuronEngine
                }

                ResistanceControl{
                    current: passiveCurrent
                }
            }
            PropertiesItem {
                text: "Synaptic input"
                info: "Es: " + (neuronEngine.synapticPotential * 1e3).toFixed(1) + " mV, " +
                      "Ts: " + (neuronEngine.synapticTimeConstant* 1e3).toFixed(1) + " ms, "+
                      "Tr: " + (neuronEngine.refractoryPeriod * 1e3).toFixed(1) + " ms, "
                SynapticPotentialControl{
                    engine: neuronEngine
                }
                SynapticTimeConstantControl{
                    engine: neuronEngine
                }
                RefractoryPeriodControl{
                    engine: neuronEngine
                }
            }
            PropertiesItem {
                text: "Synaptic output"
                info: "Gs: " + (neuronEngine.fireOutput * 1e6).toFixed(1) + " uS, "
                SynapticOutputControl {
                    engine: neuronEngine
                }
            }
            PropertiesItem {
                text: "Reset"
                RestPotentialControl{
                    engine: neuronEngine
                }
            }
        }
    }

    savedProperties: PropertyGroup {
        property alias label: neuronRoot.label
        property alias resistance: passiveCurrent.resistance
    }

}

