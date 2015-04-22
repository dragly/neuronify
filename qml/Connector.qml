import QtQuick 2.0
import "paths"

Item {
    id: root
    signal dropped(var connector)

    property point attachmentPoint: Qt.point(parent.width / 2, parent.height / 2)
    property point initialPoint: Qt.point(parent.width - connector.width / 2, parent.height - connector.height / 2)

    property alias connectorWidth: connector.width
    property alias connectorHeight: connector.height

    SCurve {
        id: connectorCurve
        z: -10
        color: "#4292c6"
        startPoint: root.attachmentPoint
        endPoint: Qt.point(connector.x + connector.width / 2, connector.y + connector.width / 2)
    }

    Item {
        id: connector

        Component.onCompleted: {
            resetPosition()
        }

        function resetPosition() {
            connector.x = root.initialPoint.x
            connector.y = root.initialPoint.y
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
                root.dropped(connector)
                connector.resetPosition()
            }
        }
    }
}
