import QtQuick 2.0
import "../style"

Item {
    id: root
    property string objectName: "CreationItem"

    signal dropped(var fileUrl, var properties, var controlParent)
    signal clicked(var entity)

    default property alias subChildren: creationControl.children

    property string name: ""
    property string description: ""
    property url source: ""
    property url imageSource: ""

    width: Style.touchableSize
    height: width

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

        Image {
            anchors.fill: parent
            source: imageSource
            fillMode: Image.PreserveAspectFit
            antialiasing: true
            smooth: true
        }

        MouseArea {
            id: dragArea
            anchors.fill: parent
            drag.target: parent
            drag.onActiveChanged: {
                if (!dragArea.drag.active) {
                    var properties = {x: creationControl.x, y: creationControl.y}
                    dropped(source, properties, root)
                }
            }
            onClicked: {
                root.clicked(root)
            }
        }
    }
}

