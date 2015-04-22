import QtQuick 2.0
import "paths"
import "hud"
import Neuronify 1.0

Node {
    id: root
    objectName: "neuron"
    fileName: "Neuron.qml"

    signal droppedConnector(var poissonGenerator, var connector)

    property point connectionPoint: Qt.point(x + width / 2, y + height / 2)
    property real stimulation: 4.0

    width: parent.width * 0.015
    height: width

    Rectangle {
        anchors.fill: parent
        color: "pink"
    }

    dumpableProperties: [
        "x",
        "y"
    ]

    engine: NodeEngine {
        onStep: {
            var shouldFire = (Math.random() < dt)
            if(shouldFire) {
                fire()
            }
        }
    }

    // TODO: Isn't this connected in the simulator automatically? If not, why?
    onSimulatorChanged: {
        if(simulator) {
            droppedConnector.connect(simulator.createConnectionToPoint)
        }
    }

    Connector {
        visible: root.selected
        onDropped: {
            root.droppedConnector(root, connector)
        }
    }
}
