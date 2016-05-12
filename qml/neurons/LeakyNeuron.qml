import QtQuick 2.6
import QtQuick.Controls 1.4

import Neuronify 1.0

import ".."
import "../controls"
import "../style"

Neuron {
    id: neuronRoot

    objectName: "leakyNeuron"
    filename: "neurons/LeakyNeuron.qml"
    imageSource: "qrc:/images/neurons/leaky.png"
    inhibitoryImageSource: "qrc:/images/neurons/leaky_inhibitory.png"

    property bool fakeInhibitory: neuronEngine.fakeFireOutput < 0

    onFakeInhibitoryChanged: {
        if(fakeInhibitory) {
            neuronRoot.inhibitory = true;
        }
    }

    savedProperties: PropertyGroup {
        property alias label: neuronRoot.label
    }

    engine: NeuronEngine {
        id: neuronEngine
        property real refractoryPeriod
        property real timeSinceFire: 1.0 / 0.0 // infinity

        savedProperties: PropertyGroup {
            property alias refractoryPeriod: neuronEngine.refractoryPeriod
            property alias resistance: leakyCurrent.resistance
        }

        onResettedProperties: {
            refractoryPeriod = 2.0e-3;
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

        LeakCurrent {
            id: leakyCurrent
        }
    }

    controls: Component {
        PropertiesPage {
            property string title: "Leaky neuron"
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
                text: "Membrane"
                info: "C: " + (neuronEngine.capacitance * 1e9).toFixed(1) + " nF, " +
                      "R: " + (leakyCurrent.resistance * 1e-3).toFixed(1) + " kΩ, "
                CapacitanceControl{
                    engine: neuronEngine
                }

                ResistanceControl{
                    current: leakyCurrent
                }

                Text {
                    property real timeConstant: neuronEngine.capacitance * leakyCurrent.resistance * 1e3
                    anchors {
                        left: parent.left
                        right: parent.right
                    }

                    font: Style.control.font
                    text: "With these properties, the time constant is " +
                          timeConstant.toFixed(1) + " ms." +
                          "For a neuron with surface area ... this is " +
                          "equivalent to ..."
                    wrapMode: Text.WrapAtWordBoundaryOrAnywhere
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
                text: "Synapse"
                info: (switchControl.isChecked ?
                           switchControl.checkedText : switchControl.uncheckedText)
                      + " , " + "τr: "
                      + (neuronEngine.refractoryPeriod * 1e3).toFixed(1) + " ms, "
                SwitchControl{
                    id: switchControl
                    target: neuronRoot
                    property: "inhibitory"
                    checkedText: "Inhibitory"
                    uncheckedText: "Excitatory"

                }

                RefractoryPeriodControl{
                    engine: neuronEngine
                }

            }

            ConnectMultipleControl {
                node: neuronRoot
            }

            ResetControl {
                engine: neuronEngine
            }
        }
    }

}

