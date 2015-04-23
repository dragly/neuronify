import QtQuick 2.0
import ".."
import "../paths"
import "../hud"
import Neuronify 1.0

Node {
    id: root
    objectName: "neuron"
    fileName: "neurons/Neuron.qml"

    property alias stimulation: engineObject.stimulation
    property alias voltage: engineObject.voltage
    property alias synapticConductance: engineObject.synapticConductance
    property alias restingPotential: engineObject.restingPotential
    property alias synapsePotential: engineObject.synapsePotential

    controls: Component {
        NeuronControls {
            neuron: root
            onDisconnectClicked: {
                simulatorRoot.disconnectNeuron(neuron)
            }
            onDeleteClicked: {
                root.destroy(1)
            }
        }
    }

    selected: false
    radius: width / 2
    width: parent.width * 0.015
    height: width
    color: stimulation > 0.0 ? "#6baed6" : "#e41a1c"

    dumpableProperties: [
        "x",
        "y",
        "clampCurrent",
        "clampCurrentEnabled",
        "adaptationIncreaseOnFire",
        "stimulation"
    ]

    engine: NeuronEngine {
        id: engineObject
        PassiveCurrent {
            id: passiveCurrent
        }
    }

    Rectangle {
        id: background
        anchors.fill: parent
        radius: width / 2
        color: root.color
        border.color: selected ? "#08306b" : "#2171b5"
        border.width: selected ? 4.0 : 2.0
    }

    Rectangle {
        anchors.fill: parent
        anchors.margins: 2.0
        radius: background.radius
        color: "#f7fbff"
        opacity: (engine.voltage + 100) / (150)
    }

    Connector {
    }
}
