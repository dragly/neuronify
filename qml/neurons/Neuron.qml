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
    readonly property bool inhibitory: root.engine.fireOutput < 0.0
    readonly property real voltage: root.engine.voltage
    property url imageSource
    property url inhibitoryImageSource

    radius: width / 2
    width: 60
    height: width
    color: inhibitory ? "#e41a1c" : "#6baed6"

    dumpableProperties: [
        "x",
        "y",
        "fireOutput"
    ]

    controls: Component {
        NeuronControls {
            engine: root.engine
            onDeleteClicked: {
                root.destroy(1)
            }
        }
    }

    engine: NeuronEngine {
        PassiveCurrent {
            id: passiveCurrent
        }
    }

    Image {
        anchors.fill: parent
        source: inhibitory ? inhibitoryImageSource : imageSource
        fillMode: Image.PreserveAspectFit
    }

    Rectangle {
        id: background
        anchors.fill: parent
        radius: width / 2
        color: "transparent"
        border.color: selected ? "#08306b" : "#2171b5"
        border.width: selected ? 4.0 : 2.0
    }

    Rectangle {
        anchors.fill: parent
        anchors.margins: 2.0
        radius: width * 0.5
        color: "#f7fbff"
        opacity: (voltage + 100) / (150)
    }

    Connector {
    }
}
