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

    function dump() {
        return {
            isAlias: true,
            parent: graphEngine.nodeIndex(sensor),
            childIndex: cellIndex
        }
    }

    onEdgeAdded: {
        connections.push(edge)
    }

    onEdgeRemoved: {
        connections.splice(connections.indexOf(edge), 1)
    }

    engine: NodeEngine {
        fireOutput: 200.0e-6
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

    Connector{
        visible: sensor.selected
    }
}
