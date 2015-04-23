import QtQuick 2.0
import "paths"

Item {
    id: root
    signal dropped(var connector)

    property point attachmentPoint: Qt.point(parent.width / 2, parent.height / 2)
    property point initialPoint: Qt.point(parent.width - draggable.width / 2, parent.height - draggable.height / 2)

    property alias connectorWidth: draggable.width
    property alias connectorHeight: draggable.height

    onDropped: {
        root.parent.droppedConnector(root.parent, draggable)
    }

    SCurve {
        id: curve
        z: -10
        color: "#4292c6"
        startPoint: root.attachmentPoint
        endPoint: Qt.point(draggable.x + draggable.width / 2, draggable.y + draggable.width / 2)
    }

    Item {
        id: draggable

        Component.onCompleted: {
            resetPosition()
        }

        function resetPosition() {
            draggable.x = root.initialPoint.x
            draggable.y = root.initialPoint.y
        }

        width: 32
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
                root.dropped(draggable)
                draggable.resetPosition()
            }
        }
    }
}
