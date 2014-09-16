import QtQuick 2.0
import QtQuick.Layouts 1.1

Rectangle {
    id: creationControlsRoot
    property bool revealed: false
    anchors {
        left: parent.left
        top: parent.top
        leftMargin: revealed ? 0.0 : -width
        bottom: parent.bottom
    }


    color: "#deebf7"
    border.color: "#9ecae1"
    border.width: 1.0
    width: parent.width * 0.1

    Behavior on anchors.leftMargin {
        NumberAnimation {
            duration: 350
            easing.type: Easing.InOutCubic
        }
    }

    signal createCompartment(var position)
    signal createVoltmeter(var position)

    Rectangle {
        anchors.left: parent.right
        width: 40
        height: 40
        color: "#deebf7"
        border.width: 1.0
        border.color: "#6baed6"
        MouseArea {
            anchors.fill: parent
            onClicked: {
                creationControlsRoot.revealed = !creationControlsRoot.revealed
            }
        }
    }

    ColumnLayout {
        id: layout
        function reset() {
            var oldSpacing = spacing
            spacing = 0
            spacing = oldSpacing
        }

        anchors {
            fill: parent
            margins: 10
        }
        spacing: 10
        Rectangle {
            id: compartmentCreator
            width: 60
            height: 40
            color: "#c6dbef"
            border.color: "#6baed6"
            border.width: 1.0

            Component.onCompleted: {
//                resetPosition()
            }

            function resetPosition() {
                layout.reset()
            }

            MouseArea {
                anchors.fill: parent
                drag.target: parent
                onReleased: {
                    createCompartment({x: compartmentCreator.x, y: compartmentCreator.y})
                    compartmentCreator.resetPosition()
                }
            }
        }

        Rectangle {
            id: voltmeterCreator
            width: 60
            height: 40
            color: "#deebf7"
            border.color: "#9ecae1"
            border.width: 1.0

            Component.onCompleted: {
//                resetPosition()
            }

            function resetPosition() {
                layout.reset()
            }

            MouseArea {
                anchors.fill: parent
                drag.target: parent
                onReleased: {
                    createVoltmeter({x: voltmeterCreator.x, y: voltmeterCreator.y})
                    voltmeterCreator.resetPosition()
                }
            }
        }
        Item {
            Layout.fillHeight: true
            Layout.fillWidth: true
        }
    }
}
