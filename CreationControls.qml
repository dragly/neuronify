import QtQuick 2.0
import QtQuick.Layouts 1.1

Rectangle {
    id: creationControlsRoot

    signal createNeuron(var position)
    signal createCompartment(var position)
    signal createVoltmeter(var position)

    property bool revealed: true

    anchors {
        left: parent.left
        top: parent.top
        leftMargin: revealed ? 0.0 : -width
        bottom: parent.bottom
    }

    color: "#deebf7"
    border.color: "#9ecae1"
    border.width: 1.0
    width: parent.width * 0.07

    Behavior on anchors.leftMargin {
        NumberAnimation {
            duration: 350
            easing.type: Easing.InOutCubic
        }
    }

//    Rectangle {
//        anchors.left: parent.right
//        width: 40
//        height: 40
//        color: "#deebf7"
//        border.width: 1.0
//        border.color: "#6baed6"
//        MouseArea {
//            anchors.fill: parent
//            onClicked: {
//                creationControlsRoot.revealed = !creationControlsRoot.revealed
//            }
//        }
//    }

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
            id: neuronCreator
            radius: width
            Layout.fillWidth: true
            Layout.minimumHeight: width
            color: "#c6dbef"
            border.color: "#6baed6"
            border.width: 2.0


            function resetPosition() {
                layout.reset()
            }

            Rectangle {
                anchors {
                    horizontalCenter: parent.right
                    verticalCenter: parent.bottom
                }
                width: parent.width * 0.2
                height: width
                color: "#4292c6"
                border.color: "#f7fbff"
                border.width: 1.0
                radius: width
            }

            MouseArea {
                anchors.fill: parent
                drag.target: parent
                onReleased: {
                    createNeuron({x: neuronCreator.x, y: neuronCreator.y})
                    compartmentCreator.resetPosition()
                }
            }
        }

        Rectangle {
            id: compartmentCreator
            radius: width * 0.1
            Layout.fillWidth: true
            Layout.minimumHeight: width * 0.67
            color: "#c6dbef"
            border.color: "#6baed6"
            border.width: 2.0


            function resetPosition() {
                layout.reset()
            }

            Rectangle {
                anchors {
                    horizontalCenter: parent.right
                    verticalCenter: parent.bottom
                }
                width: parent.width * 0.2
                height: width
                color: "#4292c6"
                border.color: "#f7fbff"
                border.width: 1.0
                radius: width
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

            Layout.fillWidth: true
            Layout.minimumHeight: width * 0.67
            color: "#deebf7"
            border.color: "#9ecae1"
            border.width: 1.0

            Component.onCompleted: {
//                resetPosition()
            }

            function resetPosition() {
                layout.reset()
            }

            Canvas {
                anchors.fill: parent
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.strokeStyle = "#e41a1c"
                    ctx.beginPath()
                    var w = width
                    var h = height
                    ctx.moveTo(h * 0.2, h * 0.2)
                    ctx.bezierCurveTo(w*0.5, h*0.2, w*0.5, h*0.8, w - h*0.2, h*0.8)
                    ctx.stroke()
                }
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
