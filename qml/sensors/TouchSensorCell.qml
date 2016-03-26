import QtQuick 2.0

import Neuronify 1.0

import "../paths"
import "../hud"
import ".."

Node {
    id: cell
    property string objectName: "touchSensorCell"

    property var sensor
    property int cellIndex: 0
    property bool sensing: false
    property real voltage: 0.0
    property real timeSinceFire: 0.0
    property bool firedLastTime: false
    property real gs: 0.0
    property real dt: 0
    property var connections: []

    useDefaultMouseHandling: false

    width: 100
    height: 100

    color: cell.sensing ? "#80e5ff" : "#0088aa"
    connectionPoint: Qt.point(sensor.x + cell.x + cell.width / 2,
                              sensor.y + cell.y + cell.height)

    onEdgeAdded: {
        connections.push(edge)
        edge.customDump = function(index, graphEngine) {
            var toNeuron = edge.itemB
            var indexOfToNeuron = graphEngine.nodeIndex(toNeuron)
            var sensorName = "entity" + graphEngine.nodeIndex(sensor)
            return "connectEntities(" + sensorName + ".cellAt(" + cellIndex + "), entity" + indexOfToNeuron + ")\n"
        }
    }

    onEdgeRemoved: {
        connections.splice(connections.indexOf(edge), 1)
    }

    engine: NodeEngine {
        fireOutput: 2.0
        onStepped: {
            if(sensing) {
                currentOutput = sensor.sensingCurrentOutput
            } else {
                currentOutput = 0.0
            }
        }
    }

    Rectangle {
        anchors.fill: parent
        color: parent.color
        border.width: cell.sensing ? width * 0.03 : width * 0.02
        border.color: "#f7fbff"
    }

    Component.onCompleted: {
        droppedConnector.connect(sensor.dropFunction)
    }

    SCurve {
        id: connectorCurve
        visible: sensor.selected
        z: -1
        color: "#0088aa"
        startPoint: Qt.point(parent.width / 2, parent.height / 2)
        endPoint: Qt.point(connector.x + connector.width / 2, connector.y + connector.width / 2)
    }

    Item {
        id: connector

        visible: sensor.selected

        Component.onCompleted: {
            resetPosition()
        }

        function resetPosition() {
            connector.x = parent.width / 2 - width / 2
            connector.y = parent.height - height / 2
        }

        width: parent.width * 0.3
        height: width

        Rectangle {
            id: connectorCircle
            anchors.centerIn: parent
            width: parent.width / 1.1
            height: width
            color: "#0088aa"
            border.color: "#80e5ff"
            border.width: 1.0
            radius: width
        }

        MouseArea {
            id: connectorMouseArea
            anchors.fill: parent
            drag.target: parent
            onReleased: {
                cell.droppedConnector(cell, connector)
                connector.resetPosition()
            }
        }
    }
}
