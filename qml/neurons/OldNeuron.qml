import QtQuick 2.0
import "../paths"
import "../hud"
import ".."
import Neuronify 1.0

Node {
    id: root
    objectName: "neuron"
    fileName: "neurons/OldNeuron.qml"

    property alias stimulation: engine.fireOutput
    property alias voltage: engine.voltage
    property real acceleration: 0.0
    property real speed: 0.0
    property alias cm: engine.cm
    //    property var outputNeurons: []
    property point connectionPoint: Qt.point(x + width / 2, y + height / 2)
    property alias synapticConductance: engine.synapticConductance
    property alias restingPotential: engine.restingPotential
    property alias synapsePotential: engine.synapsePotential
    property alias clampCurrent: engine.clampCurrent
    property alias clampCurrentEnabled: engine.clampCurrentEnabled
    property bool shouldFireOnOutput: false

    controls: Component {
        OldNeuronControls {
            neuron: root
            onDisconnectClicked: {
                simulatorRoot.disconnectNeuron(neuron)
            }
            onDeleteClicked: {
                root.destroy(1)
            }
        }
    }

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
        id: engine

        PassiveCurrent {
            id: passiveCurrent
        }

        AdaptationCurrent {
            id: adaptationCurrent
        }

        Current {
            id: currentClampCurrent
            enabled: clampCurrentEnabled
            current: clampCurrent
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
        opacity: (voltage + 100) / (150)
    }

    Connector {
        visible: root.selected

        initialPoint: Qt.point(root.width / 2 + 0.707*root.radius - connectorWidth / 2,
                               root.height / 2 + 0.707*root.radius - connectorHeight / 2)

        onDropped: {
            root.droppedConnector(root, connector)
        }
    }
}
