import QtQuick 2.0
import ".."
import "../paths"
import "../hud"
import Neuronify 1.0

Node {
    id: root
    objectName: "neuron"
    fileName: "neurons/Neuron.qml"

    readonly property real fireOutput: root.engine.fireOutput
    readonly property real voltage: root.engine.voltage

    controls: Component {
        NeuronControls {
            engine: root.engine
            onDeleteClicked: {
                root.destroy(1)
            }
        }
    }

    radius: width / 2
    width: 60
    height: width
    color: fireOutput > 0.0 ? "#6baed6" : "#e41a1c"

    dumpableProperties: [
        "x",
        "y",
        "clampCurrent",
        "clampCurrentEnabled",
        "adaptationIncreaseOnFire",
        "stimulation"
    ]

    engine: NeuronEngine {
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
        opacity: (voltage + 100) / (150)
    }

    Connector {
    }
}
