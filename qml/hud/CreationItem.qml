import QtQuick 2.0
import "../style"

MouseArea {
    id: root
    property string objectName: "CreationItem"

    signal dropped(var fileUrl, var properties, var controlParent)

    default property alias subChildren: creationControl.children

    property string name: ""
    property string description: ""
    property url source: ""
    property url imageSource: ""
    property Item parentWhenDragging
    property bool dragActive: false

    width: Style.touchableSize
    height: width

    drag.target: creationControl
    drag.threshold: 0
    onReleased: {
        console.log(creationControl.Drag.target)
        creationControl.Drag.drop()
    }

//    drag.smoothed: false
//    drag.onActiveChanged: {
//        if (!root.drag.active) {
//            var properties = {x: creationControl.x, y: creationControl.y}
//            dropped(source, properties, root)
//        }
//    }

    Item {
        id: creationControl

        property alias creationItem: root

        anchors.horizontalCenter: parent.horizontalCenter
        anchors.verticalCenter: parent.verticalCenter

        width: root.width
        height: root.height

        Drag.hotSpot.x: 32
        Drag.hotSpot.y: 32
        Drag.active: root.drag.active
        Drag.keys: ["lol"]

        states: State {
            when: root.drag.active
            ParentChange { target: creationControl; parent: root.parentWhenDragging }
            AnchorChanges { target: creationControl; anchors.horizontalCenter: undefined; anchors.verticalCenter: undefined }
            PropertyChanges {
                target: root
                dragActive: true
            }
        }

        Image {
            anchors.fill: parent
            source: imageSource
            fillMode: Image.PreserveAspectFit
            antialiasing: true
            smooth: true
            asynchronous: true
        }
    }
}

