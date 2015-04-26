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

    ListModel {
        id: categories
        ListElement {
            listSource: "NeuronList.qml"
            imageSource: "qrc:/images/categories/neuron.png"
        }

        ListElement  {
            listSource: "InhibitoryNeuronList.qml"
            imageSource: "qrc:/images/categories/inhibitory_neuron.png"
        }

        ListElement  {
            listSource: "MetersList.qml"
            imageSource: "qrc:/images/categories/meters.png"
        }

        ListElement  {
            listSource: "GeneratorsList.qml"
            imageSource: "qrc:/images/categories/generators.png"
        }

        ListElement  {
            listSource: "SensorsList.qml"
            imageSource: "qrc:/images/categories/sensors.png"
        }
    }

    MouseArea {
        anchors.fill: parent
        enabled: root.revealed
        propagateComposedEvents: true
        onPressed: {
            root.revealed = false
            mouse.accepted = false
        }
    }

    MouseArea {
        anchors.fill: creationColumn
        enabled: root.revealed
    }

    Item {
        id: background
        anchors.fill: creationColumn

        ShaderEffectSource {
            id: effectSource
            sourceItem: blurSource
            sourceRect: Qt.rect(creationColumn.x, creationColumn.y, background.width, background.height)
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
            right: creationColumn.right
            top: creationColumn.top
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

    Item {
        id: creationColumn

        anchors {
            left: parent.right
        }

        width: parent.width
        height: column.height + column.anchors.margins * 2

        Column {
            id: column

            anchors {
                top: parent.top
                left: parent.left
                right: parent.right
                margins: Style.touchableSize * 0.5
            }

            spacing: Style.touchableSize * 0.5

            ListView {
                id: categoriesListView
                height: Style.touchableSize
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
                    source: "qrc:/images/categories/marker.png"

                    width: Style.touchableSize
                    height: width
                }
            }

            Loader {
                id: loader
                anchors.horizontalCenter: parent.horizontalCenter

                height: Style.touchableSize
                source: "qrc:/qml/hud/NeuronList.qml"
            }


            states: State {
                when: root.revealed
                AnchorChanges {
                    target: creationColumn
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
}
