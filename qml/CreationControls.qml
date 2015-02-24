import QtQuick 2.0
import QtQuick.Layouts 1.1
import QtQuick.Controls 1.1

Rectangle {
    id: creationControlsRoot

    signal droppedEntity(var fileUrl, var position, var useAutoLayout)
    signal deleteEverything()

    property bool revealed: true
    property alias autoLayout: autoLayoutCheckbox.checked

    anchors {
        right: parent.right
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
            voltmeterCreator.x = 0
            voltmeterCreator.y = neuronCreator.y + neuronCreator.height + layout.spacing
        }

        anchors {
            fill: parent
            margins: 10
        }
        spacing: 10

        CreationItem {
            id: neuronCreator
            width: parent.width * 0.7
            height: width

            Rectangle {
                anchors.fill: parent
                color: "#c6dbef"
                border.color: "#6baed6"
                border.width: 2.0
                radius: width
            }

            onDropped: {
                droppedEntity(Qt.resolvedUrl("/Neuron.qml"), {x: drop.x, y: drop.y}, true)
            }
        }

        CreationItem {
            id: voltmeterCreator
            width: parent.width * 0.7
            height: width * 0.67
            Rectangle {
                anchors.fill: parent
                color: "#deebf7"
                border.color: "#9ecae1"
                border.width: 1.0

                Canvas {
                    id: canvas
                    anchors.fill: parent
                    onPaint: {
                        var ctx = getContext("2d")
                        ctx.strokeStyle = "#e41a1c"
                        ctx.beginPath()
                        var w = width
                        var h = height
                        console.log(w + " " + h)
                        //                    ctx.moveTo(h * 0.2, h * 0.2)
                        //                    ctx.bezierCurveTo(w*0.5, h*0.2, w*0.5, h*0.8, w - h*0.2, h*0.8)

                        ctx.moveTo(w*0.1, h*0.2)
                        ctx.bezierCurveTo(w*0.5, h*0.2, w*0.5, h*0.8, w*0.9, h*0.8)
                        ctx.stroke()
                    }
                }
            }

            onDropped: {
                droppedEntity(Qt.resolvedUrl("/Voltmeter.qml"), {x: drop.x, y: drop.y}, false)
            }
        }

        CreationItem {
            id: touchSensorCreator
            width: parent.width * 0.7
            height: width
            Rectangle {
                anchors.fill: parent

                color: "#4292c6"
                border.width: width * 0.02
                border.color: "#f7fbff"
            }

            onDropped: {
                droppedEntity(Qt.resolvedUrl("/TouchSensor.qml"), {x: drop.x, y: drop.y}, false)
            }
        }

        CheckBox {
            id: autoLayoutCheckbox
            text: "Auto"
            checked: true
        }

        Item {
            Layout.fillHeight: true
            Layout.fillWidth: true
        }
    }
}
