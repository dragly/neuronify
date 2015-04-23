import QtQuick 2.0

Item {
    id: creationControlBackground
    signal dropped(var position)
    default property alias subChildren: creationControl.children

    property url source: ""

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
                    dropped({x: creationControl.x + creationControlBackground.x,
                                y: creationControl.y + creationControlBackground.y})
                }
            }
        }
    }
}

