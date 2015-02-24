import QtQuick 2.0
import "paths"
import "hud"

Entity {
    id: cell
    property string objectName: "touchSensorCell"
    signal droppedConnector(var neuron, var connector)

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

    color: cell.sensing ? "#9ecae1" : "#4292c6"
    connectionPoint: Qt.point(sensor.x + cell.x + cell.width / 2,
                              sensor.y + cell.y + cell.height)

    onConnectionAdded: {
        connections.push(connection)
        connection.customDump = function(index, entities) {
            var toNeuron = connection.itemB
            var indexOfToNeuron = entities.indexOf(toNeuron)
            var sensorName = "entity" + entities.indexOf(sensor)
            return "connectEntities(" + sensorName + ".cellAt(" + cellIndex + "), entity" + indexOfToNeuron + ")\n"
        }
    }

    onConnectionRemoved: {
        connections.splice(connections.indexOf(connection), 1)
    }

    onStep: {
        cell.dt = dt
    }

    onOutputConnectionStep: {
        var neuron = target
        timeSinceFire += dt
        var V = voltage
        var Is = 0
        if(sensing) {
            gs += 20.0 * dt
        }
        Is = gs * (V - 60)
        var voltageChange = - (V + 50) - Is
        var dV = voltageChange * dt
        voltage += dV;
        if(firedLastTime) {
            voltage = -100
            gs = 0
            firedLastTime = false
            return
        }

        var shouldFire = false
        if(voltage > 0.0) {
            shouldFire = true
        }
        if(shouldFire) {
            voltage += 100.0
            timeSinceFire = 0.0
            firedLastTime = true
            neuron.stimulate(3.0)
        }
    }

    width: 100
    height: 100

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
        color: "#4292c6"
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
            width: parent.width / 2.0
            height: width
            color: "#4292c6"
            border.color: "#f7fbff"
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
