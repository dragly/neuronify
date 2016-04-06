import QtQuick 2.0
import "paths"

Item {
    id: root
    signal dropped(var connector)

    property Node _parent: parent

    property point attachmentPoint: Qt.point(parent.width / 2, parent.height / 2)
    property point initialPoint: Qt.point(parent.width, parent.height)
    property point offset: Qt.point(0.0, 0.0)

    property alias connectorWidth: draggable.width
    property alias connectorHeight: draggable.height

    property color curveColor: "#4292c6"
    property color connectorColor: "#4292c6"

    visible: _parent.selected

    onDropped: {
        root.parent.droppedConnector(root.parent, draggable)
    }

    SCurve {
        id: curve
        color: curveColor
        parent: root.parent
        z: -1
        startPoint: root.attachmentPoint
        endPoint: Qt.point(draggable.x + draggable.width / 2, draggable.y + draggable.width / 2)
        visible: root.visible
    }

    Item {
        id: draggable

        Component.onCompleted: {
            resetPosition();
            _parent.onWidthChanged.connect(resetPosition);
            _parent.onHeightChanged.connect(resetPosition);
        }

        function resetPosition() {
            draggable.x = root.initialPoint.x
            draggable.y = root.initialPoint.y
        }

        width: 64
        height: width

        Rectangle {
            id: connectorCircle
            anchors {
                centerIn: parent
            }
            width: parent.width * 0.4
            height: width
            color: connectorColor
            border.color: "#f7fbff"
            border.width: 1.0
            radius: width
        }

        MouseArea {
            id: connectorMouseArea
            anchors.fill: parent
            drag.target: parent
            onClicked: {
                root.parent.clickedConnector(parent, mouse)
            }
            onReleased: {
                if(drag.active) {
                    root.dropped(draggable)
                }
                draggable.resetPosition()
            }
        }
    }
}
