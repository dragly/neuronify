import QtQuick 2.0

Item {
    signal clicked(var entity, var mouse)
    signal dragStarted
    signal connectionAdded(var connection)
    signal connectionRemoved(var connection)

    property real radius: 1.0
    property string objectName: "entity"
    property bool selected: false
    property vector2d velocity
    property bool dragging: false
    property var copiedFrom
    property color color: "black"
    property point connectionPoint: Qt.point(x + width / 2, y + height / 2)
    property var connections: []
    property Component controls

    function addConnection(connection) {
        connections.push(connection)
        connectionAdded(connection)
    }

    function removeConnection(connection) {
        var index = connections.indexOf(connection)
        if(index > -1) {
            connections.splice(index, 1)
        }
        connectionRemoved(connection)
    }
}

