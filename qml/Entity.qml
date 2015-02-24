import QtQuick 2.0

Item {
    id: entityRoot
    signal clicked(var entity, var mouse)
    signal dragStarted
    signal connectionAdded(var connection, var isSource)
    signal connectionRemoved(var connection)
    signal aboutToDie(var entity)
    signal stimulate(var stimulation)
    signal inputConnectionStep(var source)
    signal outputConnectionStep(var target)
    signal step(var dt)
    signal finalizeStep(var dt)

    property string objectName: "entity"
    property string fileName: "Entity.qml"
    property real radius: 1.0
    property bool selected: false
    property vector2d velocity
    property bool dragging: false
    property var copiedFrom
    property color color: "black"
    property point connectionPoint: Qt.point(x + width / 2, y + height / 2)
    property Component controls
    property Item simulator
    property bool useDefaultMouseHandling: true
    property var dumpableProperties: [
        "x",
        "y"
    ]

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

    function dump(index, entities) {
        var outputString = ""
        outputString += _basicSelfDump(index)
        return outputString
    }

    Component.onDestruction: {
        aboutToDie(entityRoot)
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

