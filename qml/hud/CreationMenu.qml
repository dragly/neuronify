import QtQuick 2.0
import QtQuick.Layouts 1.1
import QtQuick.Controls 1.1
import QtGraphicalEffects 1.0

import "../neurons"
import "../style"

Item {
    id: root

    signal droppedEntity(var fileUrl, var properties, var controlParent, var useAutoLayout)
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

        ListModel {
            id: categories
            ListElement {
                listSource: "qrc:/qml/hud/NeuronList.qml"
                imageSource: "qrc:/images/creators/categories/neuron.png"
            }

            ListElement  {
                listSource: "qrc:/qml/hud/InhibitoryNeuronList.qml"
                imageSource: "qrc:/images/creators/categories/inhibitory_neuron.png"
            }

            ListElement  {
                listSource: "qrc:/qml/hud/MetersList.qml"
                imageSource: "qrc:/images/creators/categories/meters.png"
            }
        }

        Column {
            anchors {
                fill: parent
                margins: Style.touchableSize * 0.5
            }

            ListView {
                id: categoriesListView
                height: parent.height / 2
                width: count * (Style.touchableSize + spacing) - spacing
                anchors.horizontalCenter: parent.horizontalCenter

                spacing: Style.touchableSize * 0.5

                orientation: ListView.Horizontal

                model: categories

                delegate: Image {
                    width: Style.touchableSize
                    height: width
                    source: model.imageSource

                    MouseArea {
                        anchors.fill: parent
                        onClicked: {
                            categoriesListView.currentIndex = index
                            loader.source = model.listSource
                        }
                    }
                }

                highlight: Image {
                    source: "qrc:/images/creators/categories/marker.png"

                    width: Style.touchableSize
                    height: width
                }
            }

            Loader {
                id: loader
                anchors.horizontalCenter: parent.horizontalCenter
                height: parent.height / 2
                source: "qrc:/qml/hud/NeuronList.qml"
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
