import QtQuick 2.0

Item {
    id: entityRoot
    signal clicked(var entity, var mouse)
    signal dragStarted
    signal connectionAdded(var connection)
    signal connectionRemoved(var connection)
    signal aboutToDie(var entity)
    signal stimulated(var stimulation)
    signal inputConnectionStep(var source)
    signal outputConnectionStep(var target)

    property string objectName: "entity"
    property string fileName: "Entity.qml"
    property real radius: 1.0
    property bool selected: false
    property vector2d velocity
    property bool dragging: false
    property var copiedFrom
    property color color: "black"
    property point connectionPoint: Qt.point(x + width / 2, y + height / 2)
    property var connections: []
    property Component controls
    property Item simulator
    property bool useDefaultMouseHandling: true
    property var dumpableProperties: [
        "x",
        "y"
    ]

    function stimulate(stimulation) {
        stimulated(stimulation)
    }

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

    function _basicSelfDump(index) {
        var outputString = ""
        var entityData = {}

        for(var i in dumpableProperties) {
            var propertyName = dumpableProperties[i]
            entityData[propertyName] = entityRoot[propertyName]
        }

        var entityName = "entity" + index
        var ss = "var " + entityName + " = createEntity(\"" + fileName + "\", " + JSON.stringify(entityData) + ")"
        outputString += ss + "\n"
        return outputString
    }

    function _basicConnectionDump(index, entities) {
        var outputString = ""
        for(var j in connections) {
            var targetEntity = connections[j].itemB
            var targetEntityIndex = entities.indexOf(targetEntity)
            outputString += "connectEntities(entity" + index + ", entity" + targetEntityIndex + ")\n"
        }
        return outputString
    }

    function dump(index, entities) {
        var outputString = ""
        outputString += _basicSelfDump(index)
        outputString += _basicConnectionDump(index, entities)
        return outputString
    }

    function stepForward(dt) {

    }

    function finalizeStep(dt) {

    }

    Component.onDestruction: {
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

