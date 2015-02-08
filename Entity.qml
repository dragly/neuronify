import QtQuick 2.0

Item {
    property string objectName: "entity"
    property bool selected: false
    signal clicked(var neuron, var mouse)
    signal dragStarted
    property vector2d velocity
    property bool dragging: false
    property var copiedFrom
    property color color: "black"
    property point connectionPoint: Qt.point(x + width / 2, y + height / 2)
    property var connections: []

    function addConnection(connection) {
        connections.push(connection)
    }

    function removeConnection(connection) {
        var index = connections.indexOf(connection)
        if(index > -1) {
            connections.splice(index, 1)
        }
    }
}

