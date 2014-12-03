import QtQuick 2.0
import QtQuick.Layouts 1.1
import QtQuick.Controls 1.1

Rectangle {
    id: creationControlsRoot

    signal createNeuron(var position)
    signal createCompartment(var position)
    signal createVoltmeter(var position)
    signal deleteEverything()

    property bool revealed: true
    property alias autoLayout: autoLayoutCheckbox.checked
    property alias deletaAllButton1: deleteAllButton
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

    Column {
        id: layout

        function reset() {
            neuronCreator.x = 0
            neuronCreator.y = 0
            compartmentCreator.x = 0
            compartmentCreator.y = neuronCreator.y + neuronCreator.height + layout.spacing
            voltmeterCreator.x = 0
            voltmeterCreator.y = compartmentCreator.y + compartmentCreator.height + layout.spacing
        }

        anchors {
            fill: parent
            margins: 10
        }
        spacing: 10

        Rectangle {
            id: neuronCreator
            radius: width
            width: parent.width
            height: width
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
            width: parent.width
            height: width * 0.67
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

            width: parent.width
            height: width * 0.67
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
        CheckBox {
            id: autoLayoutCheckbox
            text: "Auto"
            checked: true
        }

        Button {
            id: deleteAllButton

            text: "Delete All"
            onClicked: {
                deleteEverything()
            }
        }

        Button {
            id: saveButton

            text: "Save State"
            onClicked: {

                saveState()
            }
        }
        Button {
            id: loadButton

            text: "Load State"
            onClicked: loadFileDialog.visible = true
        }

        Item {
            Layout.fillHeight: true
            Layout.fillWidth: true
        }
    }
}
