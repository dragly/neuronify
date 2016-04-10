import QtQuick 2.0
import Neuronify 1.0

import "qrc:/"
import "qrc:/qml/"
import "qrc:/qml/controls"
import "../style"

Neuron {
    id: neuronRoot

    objectName: "BurstNeuron"
    filename: "neurons/BurstNeuron.qml"
    imageSource: "qrc:/images/neurons/burst.png"
    inhibitoryImageSource: "qrc:/images/neurons/burst_inhibitory.png"

    engine: NeuronEngine {
        id: neuronEngine
        PassiveCurrent {
            id: passiveCurrent
        }
        Current {
            property real boost
            property real maximumBoost: 1.0e-9
            onFired: {
                if(boost < 1.0e-12) {
                    boost = 1.2e-9
                }
            }
            onStepped: {
                if(boost > 0.0) {
                    boost = boost - 6.0*maximumBoost*dt
                } else {
                    boost = 0.0
                }
                current = boost
            }
            onResettedDynamics: {
                boost = 0.0;
            }
        }
    }

    controls: Component {
        PropertiesPage {
            property string title: "Burst neuron"
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
            Text {
                text: "Fixed parameters: "
                font: Style.control.font
                color: Style.text.color
            }

            Text {
                id: subText
                font: Style.control.subText.font
                color: Style.control.subText.color
                text: "Vr: " +
                      (neuronEngine.restingPotential * 1e3).toFixed(1)
                      + " mV, " +
                      "Vi: " +
                      (neuronEngine.initialPotential * 1e3).toFixed(1)
                      + " mV, " +
                      "Vt: " +
                      (neuronEngine.threshold * 1e3).toFixed(1)
                      + " mV \n" +
                      "C: " +
                      (neuronEngine.capacitance * 1e9).toFixed(1)
                      + " nF, " +
                      "R: " +
                      (passiveCurrent.resistance * 1e-3).toFixed(1)
                      + " kΩ, \n" +
                      "τr: "+
                      0
                      + " ms, "
            }
            spacing: 10
            RestPotentialControl{
                engine: neuronEngine
            }
        }
    }
}

