import QtQuick 2.0
import QtQuick.Layouts 1.1
import QtQuick.Controls 1.1
import QtGraphicalEffects 1.0

import "../neurons"
import "../style"

Item {
    id: root

    signal droppedEntity(var fileUrl, var properties, var controlParent)
    signal deleteEverything()

    property var blurSource: null
    property bool revealed: false

    width: parent.width * 0.1
    anchors.fill: parent

    onRevealedChanged: {
//        itemListView.currentIndex = -1
        itemListView.currentIndex = 0
    }

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
        ListElement  {
            listSource: "AnnotationsList.qml"
            imageSource: "qrc:/images/categories/annotate.png"
        }
    }

    Loader {
        id: itemModelLoader
        source: "NeuronList.qml"
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


    Item {
        id: background
        anchors.fill: creationColumn

        Rectangle {
            anchors.fill: parent
            color: Style.color.background
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
                boundsBehavior: Flickable.StopAtBounds

                model: categories

                delegate: Image {
                    width: Style.touchableSize
                    height: width
                    source: model.imageSource

                    MouseArea {
                        anchors.fill: parent
                        onPressed: {
                            categoriesListView.currentIndex = index
                            itemModelLoader.source = model.listSource
                            itemListView.currentIndex = 0

                        }
                    }
                }

                highlight: Image {
                    source: "qrc:/images/categories/marker.png"

                    width: Style.touchableSize
                    height: width
                }
            }

            ListView {
                id: itemListView
                height: Style.touchableSize
                width: count * (Style.touchableSize + spacing) - spacing
                anchors.horizontalCenter: parent.horizontalCenter

                spacing: Style.touchableSize * 0.5
                orientation: ListView.Horizontal
                boundsBehavior: Flickable.StopAtBounds

                model: itemModelLoader.item


                delegate: CreationItem {
                    name: model.name
                    description: model.description
                    source: model.source
                    imageSource: model.imageSource


                    onPressed: {
                        itemListView.currentIndex = index
                    }

                    onDropped: {
                        droppedEntity(fileUrl, properties, controlParent)
                    }
                }

                highlight: Image {
                    z: 99
                    source: "qrc:/images/categories/marker.png"

                    opacity: 0.6

                    width: Style.touchableSize
                    height: width
                }

                onModelChanged: {
                    currentIndex = -1
                }
            }

            Item {
                id: itemDescription

                anchors.horizontalCenter: parent.horizontalCenter

                width: parent.width
                height: 0.0
                anchors.top: itemListView.bottom

                Text {
                    property var item: itemListView.currentItem
                    anchors.fill: parent

                    color: Style.text.color
                    font: Style.text.font
                    horizontalAlignment: Text.AlignHCenter
                    wrapMode: Text.WrapAtWordBoundaryOrAnywhere

                    text: item ? "<b>" + item.name + "</b> - " + item.description : ""
                }

                states: State {
                    when: itemListView.currentItem !== null
                    PropertyChanges {
                        target: itemDescription
                        height: Style.font.size * 1.1
                    }
                }
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
