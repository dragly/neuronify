import QtQuick 2.0
import "paths"
import "hud"
import Neuronify 1.0

Node {
    id: root
    objectName: "neuron"
    fileName: "Neuron.qml"

    property point connectionPoint: Qt.point(x + width / 2, y + height / 2)
    property real stimulation: 4.0

    width: parent.width * 0.015
    height: width

    dumpableProperties: [
        "x",
        "y"
    ]

    engine: NodeEngine {
        stimulation: 4.0
        onStepped: {
            var shouldFire = (Math.random() < dt)
            if(shouldFire) {
                console.log("Fired poisson")
                fire()
            }
        }
    }

    Rectangle {
        anchors.fill: parent
        color: "pink"
    }

    Connector {
        visible: root.selected
        onDropped: {
            root.droppedConnector(root, connector)
        }
    }
}
