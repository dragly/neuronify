import QtQuick 2.6
import QtQuick.Controls 1.4

import Neuronify 1.0

import "qrc:/"
import "qrc:/qml/"
import "qrc:/qml/controls"



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
        Column {
            property StackView stackView: Stack.view
            ListView {
                model: ListModel {
                    ListElement {
                        name: "Dynamics"
                    }
                }
                delegate: Item {
                    width: parent.width
                    height: buttonLabel.height
                    Label {
                        id: buttonLabel
                        anchors {
                            left: parent.left
                            leftMargin: Style.spacing
                            verticalCenter: parent.verticalCenter
                        }
                        text: name
                    }
                    Image {
                        anchors {
                            right: parent.right
                            verticalCenter: parent.verticalCenter
                            rightMargin: Style.spacing
                        }
                        source: "qrc:/images/back.png"
                        rotation: 180
                    }
                    MouseArea {
                        anchors.fill: parent
                        onClicked: {
                            console.log("Push " + name)
                        }
                    }
                }
            }
            Component {
                id: comp
                Column {
                    RestingPotentialControl{
                        engine: neuronEngine
                    }
                }
            }
            LabelControl {
                neuron: neuronRoot
            }

            RestingPotentialControl{
                engine: neuronEngine
            }

            InitialPotentialControl{
                engine: neuronEngine
            }

            ThresholdControl{
                engine: neuronEngine
            }

            CapacitanceControl{
                engine: neuronEngine
            }

            ResistanceControl{
                current: passiveCurrent
            }

            RefractoryPeriodControl{
                engine: neuronEngine
            }


            SynapticOutputControl {
                engine: neuronEngine
            }


            SynapticPotentialControl{
                engine: neuronEngine
            }

            SynapticTimeConstantControl{
                engine: neuronEngine
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
        property alias resistance: passiveCurrent.resistance
        property alias capacitance: neuronEngine.capacitance
        property alias refractoryPeriod: neuronEngine.refractoryPeriod
        property alias restingPotential: neuronEngine.restingPotential
        property alias initialPotential: neuronEngine.initialPotential
        property alias threshold: neuronEngine.threshold
        property alias synapticTimeConstant: neuronEngine.synapticTimeConstant
        property alias synapticPotential: neuronEngine.synapticPotential

    }

}

