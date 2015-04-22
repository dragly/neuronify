import QtQuick 2.0
import "paths"
import "hud"
import Neuronify 1.0

Entity {
    id: root
    objectName: "neuron"
    fileName: "Neuron.qml"

    signal droppedConnector(var poissonGenerator, var connector)

    property point connectionPoint: Qt.point(x + width / 2, y + height / 2)
    property real outputStimulation: 4.0
    property bool shouldFireOnOutput: false

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

    onStep: {
        shouldFireOnOutput = (Math.random() < dt)
    }

    onOutputConnectionStep: {
        if(shouldFireOnOutput) {
            target.stimulate(outputStimulation)
        }
    }

    onStimulate: {
        engine.stimulate(stimulation)
    }

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
