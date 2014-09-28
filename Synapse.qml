import QtQuick 2.0

Item {
    id: synapseRoot

    property bool selected: false
    signal clicked(var synapse)
    signal dragStarted
    property vector2d velocity
    property bool dragging: false

    property var connections: []

    width: 100
    height: 70

    function compartmentUnderConnector(position) {
        if(position.x < 50) {
            return preSynapse
        } else {
            return postSynapse
        }
    }

    function stepForward(dt) {

    }

    function finalizeStep() {

    }

    Rectangle {
        id: preSynapse
        property bool selected: parent.selected
        property vector2d velocity
        property var connections: []

        function addConnection(connection) {
            connections.push(connection)
        }

        function removeConnection(connection) {
            var connectionIndex = connections.indexOf(connection)
            if(connectionIndex > -1) {
                connections.splice(connectionIndex, 1)
            }
        }

        anchors {
            left: parent.left
            verticalCenter: parent.verticalCenter
        }

        color: "#6baed6"
        border.color: selected ? "#08306b" : "#2171b5"
        border.width: selected ? 3.0 : 1.0
        radius: 10
        width: 48
        height: 50
    }

    Rectangle {
        id: postSynapse
        anchors {
            right: parent.right
            verticalCenter: parent.verticalCenter
        }

        radius: 10
        width: 48
        height: 70
        color: "#6baed6"
        border.color: selected ? "#08306b" : "#2171b5"
        border.width: selected ? 3.0 : 1.0
    }

    MouseArea {
        anchors.fill: parent
        drag.target: parent
        onClicked: {
            synapseRoot.clicked(synapseRoot)
        }
        onPressed: {
            synapseRoot.dragging = true
            dragStarted()
        }

        onReleased: {
            synapseRoot.dragging = false
        }
    }
}
