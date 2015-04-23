import QtQuick 2.0

Item {
    id: creationControlBackground
    signal dropped(point drop)
    default property alias subChildren: creationControl.children

    Item {
        id: creationControl

        anchors.horizontalCenter: parent.horizontalCenter
        anchors.verticalCenter: parent.verticalCenter

        width: parent.width
        height: parent.height

        states: State {
            when: dragArea.drag.active
            AnchorChanges { target: creationControl; anchors.horizontalCenter: undefined; anchors.verticalCenter: undefined }
        }

        Drag.dragType: Drag.Automatic

        MouseArea {
            id: dragArea
            anchors.fill: parent
            drag.target: parent
            drag.onActiveChanged: {
                if (!dragArea.drag.active) {
                    dropped(Qt.point(creationControl.x + creationControlBackground.x,
                                     creationControl.y + creationControlBackground.y))
                }
            }
        }
    }
}

