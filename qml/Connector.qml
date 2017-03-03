import QtQuick 2.0
import "paths"

Item {
    id: root
    signal dropped(var connector)

    property Node _parent: parent

    property point attachmentPoint: Qt.point(parent.width / 2, parent.height / 2)
    property point initialPoint: Qt.point(parent.width , parent.height)
    property point offset: Qt.point(0.0, 0.0)

    property alias connectorWidth: connectorCircle.width
    property alias connectorHeight: connectorCircle.height

    property color color: "pink"
    property color connectorColor: color

    property bool childrenVisible: (_parent.selected ||
                                   connectorMouseArea.containsMouse ||
                                   connectorMouseArea.drag.active ||
                                   parentMouseArea.containsMouse)

    MouseArea {
        id: parentMouseArea
        parent: root.parent
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.NoButton
    }

    SCurve {
        id: curve

        color: root.color
        parent: root.parent
        z: -1
        startPoint: root.attachmentPoint
        endPoint: {
            if(connectorMouseArea.drag.active) {
                return Qt.point(connectorCircle.x + connectorCircle.width / 2, connectorCircle.y + connectorCircle.height / 2)
            } else {
                return Qt.point(root.initialPoint.x + connectorMouseArea.width / 2, root.initialPoint.y + connectorMouseArea.height / 2)
            }
        }
        visible: root.childrenVisible
    }


    MouseArea {
        id: connectorMouseArea
        x: root.initialPoint.x
        y: root.initialPoint.y
        hoverEnabled: true
        width: 64
        height: width
        drag.target: connectorCircle
        onClicked: {
            root.parent.clickedConnector(parent, mouse)
        }
        onReleased: {
            connectorCircle.Drag.drop()
        }

        Rectangle {
            id: connectorCircle

            property Node node: root._parent

            visible: root.childrenVisible
            width: 24
            height: width
            anchors {
                verticalCenter: parent.verticalCenter
                horizontalCenter: parent.horizontalCenter
            }
            Drag.keys: ["connector"]
            Drag.active: connectorMouseArea.drag.active
            Drag.hotSpot.x: width / 2
            Drag.hotSpot.y: height / 2
            color: connectorColor
            border.color: "#f7fbff"
            border.width: 1.0
            radius: width

            states: State {
                when: connectorMouseArea.drag.active
                AnchorChanges { target: connectorCircle; anchors.verticalCenter: undefined; anchors.horizontalCenter: undefined }
                ParentChange { target: connectorCircle; parent: root }
            }
        }
    }
}
