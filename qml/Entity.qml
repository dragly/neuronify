import QtQuick 2.0

Item {
    id: entityRoot
    signal clicked(var entity, var mouse)
    signal dragStarted
    signal connectionAdded(var connection)
    signal connectionRemoved(var connection)
    signal aboutToDie(var entity)

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
    property bool useDefaultMouseHandling: true

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

    function _deleteAllConnectionsInList(connectionsToDelete) {
        for(var i in connectionsToDelete) {
            var connection = connectionsToDelete[i]
            connection.destroy()
        }
    }

    Component.onDestruction: {
        console.log("Destroying entity " + entityRoot)
        aboutToDie(entityRoot)
        _deleteAllConnectionsInList(connections)
    }

    MouseArea {
        enabled: useDefaultMouseHandling
        anchors.fill: parent
        drag.target: parent
        onPressed: {
            entityRoot.dragging = true
            dragStarted()
        }

        onClicked: {
            entityRoot.clicked(entityRoot, mouse)
        }

        onReleased: {
            entityRoot.dragging = false
        }
    }
}

