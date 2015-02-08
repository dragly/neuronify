import QtQuick 2.0
import "paths"

Entity {
    id: sensorRoot
    objectName: "touchSensor"
    property int cells: 5
    property var dropFunction

    width: cells * 100
    height: 100

    Component.onCompleted: {
        resetCells()
    }

    function resetCells() {
        console.log("Resetting cells!")
        for(var i = 0; i < repeater.count; i++) {
            console.log("Found cell!")
            var cell = repeater.itemAt(i)
            for(var j in cell.connections) {
                var connection = cell.connections[j]
                console.log("Deleting connection!")
                deleteConnection(connection)
            }
        }
        console.log("Changing number of cells!")
        repeater.model = cells
    }

    function stepForward(dt) {
        for(var i = 0; i < repeater.count; i++) {
            var cell = repeater.itemAt(i)
            cell.stepForward(dt)
        }
    }

    function dump(index, neurons) {
        var outputString = ""
        var sensorData = {
            x: x,
            y: y,
            cells: cells
        }
        var sensorName = "sensor" + index
        var ss = "var " + sensorName + " = createTouchSensor(" + JSON.stringify(sensorData) + ")"
        outputString += ss + "\n"

        for(var j = 0; j < repeater.count; j++) {
            var cell = repeater.itemAt(j)

            for(var k in cell.connections){
                var toNeuron = cell.connections[k].itemB
                var indexOfToNeuron = neurons.indexOf(toNeuron)
                outputString += "connectSensorToNeuron(" + sensorName + ".cellAt(" + j + "), neuron" + indexOfToNeuron + ") \n"
            }
        }
        console.log(outputString)
        return outputString
    }

    function cellAt(index) {
        var cell = repeater.itemAt(index)
        if(!cell) {
            console.warn("WARNING: No cell at index " + index)
        }
        return cell
    }

    onCellsChanged: {
        resetCells()
    }

    MouseArea {
        anchors.fill: cellRow

        function desenseAll() {
            for(var i = 0; i < repeater.count; i++) {
                var item = repeater.itemAt(i)
                item.sensing = false
            }
        }

        function senseObject(mouse) {
            desenseAll()
            var index = mouse.x / 100
            var item = repeater.itemAt(index)
            if(item) {
                item.sensing = true
            }
        }

        onPressed: {
            senseObject(mouse)
        }

        onPositionChanged: {
            senseObject(mouse)
        }

        onReleased: {
            desenseAll()
        }

        onExited: {
            desenseAll()
        }
    }

    Row {
        id: cellRow
        Repeater {
            id: repeater
            model: 0

            Entity {
                id: cell
                property string objectName: "touchSensorCell"
                signal droppedConnector(var neuron, var connector)
                property bool sensing: false

                color: cell.sensing ? "#9ecae1" : "#4292c6"
                connectionPoint: Qt.point(sensorRoot.x + cell.x + cell.width / 2,
                                          sensorRoot.y + cell.y + cell.height)

                function stepForward(dt) {
                    for(var i in connections) {
                        var connection = connections[i]
                        var neuron = connection.itemB
                        if(sensing) {
                            neuron.stimulate(0.5)
                        }
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
                    droppedConnector.connect(sensorRoot.dropFunction)
                }

                SCurve {
                    id: connectorCurve
                    visible: sensorRoot.selected
                    z: -1
                    color: "#4292c6"
                    startPoint: Qt.point(parent.width / 2, parent.height / 2)
                    endPoint: Qt.point(connector.x + connector.width / 2, connector.y + connector.width / 2)
                }

                Item {
                    id: connector

                    visible: sensorRoot.selected

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
        }
    }


    Rectangle {
        anchors {
            horizontalCenter: parent.left
            verticalCenter: parent.top
        }
        width: parent.height / 3
        height: width
        radius: width / 2
        color: "#c6dbef"
        border.width: width * 0.1
        border.color: "#f7fbff"

        Image {
            anchors.fill: parent
            anchors.margins: parent.width * 0.1
            source: "images/transform-move.png"
            smooth: true
            antialiasing: true
        }

        MouseArea {
            anchors.fill: parent
            drag.target: sensorRoot
            onPressed: {
                sensorRoot.dragging = true
                dragStarted()
            }

            onClicked: {
                sensorRoot.clicked(sensorRoot, mouse)
            }

            onReleased: {
                sensorRoot.dragging = false
            }
        }
    }
}

