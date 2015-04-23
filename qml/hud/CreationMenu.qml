import QtQuick 2.0
import QtQuick.Layouts 1.1
import QtQuick.Controls 1.1
import QtGraphicalEffects 1.0

import "../neurons"
import "../style"

Item {
    id: root

    signal droppedEntity(var fileUrl, var properties, var useAutoLayout)
    signal deleteEverything()

    property var blurSource: null
    property bool revealed: false

    width: parent.width * 0.1

    anchors.fill: parent

    MouseArea {
        anchors.fill: parent
        enabled: root.revealed
        propagateComposedEvents: true
        onPressed: {
            root.revealed = false
            mouse.accepted = false
        }
    }

    Item {
        id: layoutRect

        anchors {
            left: parent.right
            top: parent.top
        }

        width: parent.width
        height: parent.height * 0.5

        MouseArea {
            anchors.fill: parent
            enabled: root.revealed
        }

        Item {
            id: background
            anchors.fill: parent

            ShaderEffectSource {
                id: effectSource
                sourceItem: blurSource
                sourceRect: Qt.rect(layoutRect.x, layoutRect.y, background.width, background.height)
                anchors.fill: parent
            }

            FastBlur {
                anchors.fill: parent
                source: effectSource

                radius: Style.size * 6
            }

            Rectangle {
                anchors.fill: parent
                color: Qt.rgba(1.0, 1.0, 1.0, 0.6)
                border.color: Qt.rgba(0.8, 0.8, 0.8)
                border.width: 2.0
            }
        }

        Image {
            anchors {
                right: parent.right
                top: parent.top
                margins: Style.margin
            }
            width: Style.touchableSize
            height: width

            source: "qrc:/images/back.png"

            rotation: 180

            MouseArea {
                anchors.fill: parent
                onClicked: {
                    root.revealed = false
                }
            }
        }

        Row {

            anchors {
                fill: parent
                margins: Style.touchableSize * 0.5
            }

            spacing: Style.touchableSize * 0.5

            AdaptationNeuronCreator {
                onDropped: {
                    droppedEntity(source, position, true)
                }
            }

            CreationItem {
                id: poissonCreator
                width: Style.touchableSize
                height: width

                source: "qrc:/qml/generators/PoissonGenerator.qml"

                Rectangle {
                    anchors.fill: parent
                    color: "#c6dbef"
                    border.color: "#6baed6"
                    border.width: 2.0
                }

                onDropped: {
                    droppedEntity(source, position, true)
                }
            }

            CreationItem {
                width: Style.touchableSize
                height: width

                Rectangle {
                    anchors.fill: parent
                    color: "orange"
                    border.color: "#6baed6"
                    border.width: 2.0
                }

                onDropped: {
                    droppedEntity("generators/CurrentClamp.qml", {x: drop.x, y: drop.y}, true)
                }
            }

            CreationItem {
                id: voltmeterCreator
                width: Style.touchableSize
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

                            ctx.moveTo(w*0.1, h*0.2)
                            ctx.bezierCurveTo(w*0.5, h*0.2, w*0.5, h*0.8, w*0.9, h*0.8)
                            ctx.stroke()
                        }
                    }
                }

                onDropped: {
                    droppedEntity("Voltmeter.qml", {x: drop.x, y: drop.y}, false)
                }
            }

            CreationItem {
                id: touchSensorCreator
                width: Style.touchableSize
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

            Item {
                Layout.fillHeight: true
                Layout.fillWidth: true
            }
        }

        states: State {
            when: root.revealed
            AnchorChanges {
                target: layoutRect
                anchors.left: root.left
            }
        }

        transitions: Transition {
            AnchorAnimation {
                duration: 400
                easing.type: Easing.InOutQuad
            }
        }
    }
}
