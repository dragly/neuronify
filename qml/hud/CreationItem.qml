import QtQuick 2.0
import QtQuick.Controls 2.1
import "qrc:/qml/style"

Item {
    id: root
    property string objectName: "CreationItem"

    signal dropped(var fileUrl, var properties, var controlParent)

//    default property alias subChildren: creationControl.children

    property string name: "test long text with long name"
    property string description: ""
    property url source: ""
    property url imageSource: "qrc:/images/neurons/leaky.png"
    property Item parentWhenDragging
    property bool dragActive: false

    width: 64
    height: column.height

    Column {
        id: column
        anchors {
            left: parent.left
            right: parent.right
        }
        spacing: 8

        MouseArea {
            id: mouseArea

            width: parent.width
            height: width

            drag.target: creationControl
//            drag.threshold: 0

            onReleased: {
                creationControl.Drag.drop()
            }

            Item {
                id: creationControl

                property alias creationItem: root

                anchors.horizontalCenter: parent.horizontalCenter
                anchors.verticalCenter: parent.verticalCenter

                width: mouseArea.width + 16
                height: width

                Drag.hotSpot.x: 32
                Drag.hotSpot.y: 32
                Drag.active: mouseArea.drag.active
                Drag.keys: ["lol"]

                states: State {
                    when: mouseArea.drag.active
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

        Text {
            id: text
            anchors {
                left: parent.left
                right: parent.right
            }
            color: Style.creation.text.color
            font.pixelSize: 12
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WrapAtWordBoundaryOrAnywhere
            text: root.name
        }

        Item {
            width: 1
            height: 8
        }
    }
}

