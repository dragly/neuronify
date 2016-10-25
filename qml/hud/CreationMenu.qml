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
        itemListView.currentIndex = 0

        if (revealed) {
            focus = true
            forceActiveFocus()
        }
         else {
            focus = false
        }
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

//        ListElement  {
//            listSource: "SensorsList.qml"
//            imageSource: "qrc:/images/categories/sensors.png"
//        }
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

    Item {
        id: creationColumn

        anchors {
            left: parent.right
        }

        width: parent.width
        height: column.height + column.anchors.margins * 2

        MouseArea {
            anchors.fill: parent
        }

        Column {
            id: column

            anchors {
                top: parent.top
                left: parent.left
                right: parent.right
                rightMargin: backButton.width
                margins: Style.touchableSize * 0.2
            }

            spacing: Style.touchableSize * 0.2

            Row {
                id: categoriesListView

                property int currentIndex: 0

                height: Style.touchableSize
                anchors.horizontalCenter: parent.horizontalCenter

                spacing: Style.touchableSize * 0.5

                Repeater {

                    model: categories

                    Item {
                        width: Math.min(Style.touchableSize, column.width / categories.count - categoriesListView.spacing)
                        height: width

                        Image {
                            anchors {
                                fill: parent
                                margins: -parent.width * 0.05
                            }
                            source: "qrc:/images/categories/marker.png"
                            asynchronous: true
                            smooth: true
                            antialiasing: true
                            visible: index === categoriesListView.currentIndex
                        }
                        Image {
                            anchors.fill: parent
                            asynchronous: true
                            source: model.imageSource
                            smooth: true
                            antialiasing: true
                        }
                        MouseArea {
                            anchors.fill: parent
                            onPressed: {
                                categoriesListView.currentIndex = index;
                                itemModelLoader.source = model.listSource;
                                itemListView.currentIndex = 0;

                            }
                        }
                    }
                }
            }

            Row {
                id: itemListView

                property int currentIndex: 0
                property var currentItem

                function refresh() {
                    if(itemListRepeater.itemAt(currentIndex)) {
                        currentItem = itemListRepeater.itemAt(currentIndex).item;
                    } else {
                        currentItem = null;
                    }
                }

                onCurrentIndexChanged: {
                    itemListView.refresh();
                }

                height: Style.touchableSize
                width: itemModelLoader.item.count * (Style.touchableSize + spacing) - spacing
                anchors.horizontalCenter: parent.horizontalCenter

                spacing: Style.touchableSize * 0.5

                Repeater {
                    id: itemListRepeater

                    model: itemModelLoader.item

                    delegate: Item {
                        property var item: creationItem
                        width: Math.min(Style.touchableSize, column.width / itemListRepeater.count - itemListView.spacing)
                        height: width

                        Image {
                            id: markerImage
                            anchors {
                                fill: parent
                                margins: -parent.width * 0.2
                            }
                            source: "qrc:/images/categories/marker.png"
                            smooth: true
                            antialiasing: true
                            visible: index === itemListView.currentIndex
                        }

                        CreationItem {
                            id: creationItem
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
                    }

                    onModelChanged: {
                        itemListView.currentIndex = 0;
                        itemListView.refresh();
                    }
                }
            }

            Text {
                property var item: itemListView.currentItem

                anchors {
                    left: parent.left
                    right: parent.right
                }

                color: Style.text.color
                font: Style.text.font
                horizontalAlignment: Text.AlignHCenter
                wrapMode: Text.WrapAtWordBoundaryOrAnywhere

                text: item ? "<b>" + item.name + "</b> - " + item.description : ""
            }
        }

        Image {
            id: backButton
            anchors {
                right: creationColumn.right
                top: creationColumn.top
                topMargin: Style.margin
            }
            width: Style.touchableSize
            height: width

            source: "qrc:/images/tools/back.png"

            rotation: 180

            MouseArea {
                anchors.fill: parent
                onClicked: {
                    root.revealed = false
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

    Keys.onPressed: {
        console.log("caught button press CREATION")

        if(event.key === Qt.Key_Back || event.key === Qt.Key_Escape) {
            revealed = false
        }

    }
}
