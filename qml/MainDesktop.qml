import QtQuick 2.5
import QtQuick.Controls 1.4 as QQC1
import QtQuick.Controls 2.1
import QtQuick.Controls.Material 2.1
import QtQuick.Dialogs 1.0
import QtQuick.Layouts 1.3
import QtQuick.Particles 2.0
import QtQuick.Window 2.1

import QtCharts 2.1
import QtMultimedia 5.5
import Qt.labs.settings 1.0
import Qt.labs.folderlistmodel 2.1
import Qt.labs.platform 1.0

import Neuronify 1.0
import CuteVersioning 1.0
import QtGraphicalEffects 1.0

import "qrc:/qml/backend"
import "qrc:/qml/controls"
import "qrc:/qml/hud"
import "qrc:/qml/io"
import "qrc:/qml/hud"
import "qrc:/qml/menus"
import "qrc:/qml/menus/filemenu"
import "qrc:/qml/menus/mainmenu"
import "qrc:/qml/tools"
import "qrc:/qml/store"
import "qrc:/qml/style"
import "qrc:/qml/ui"

Item {
    id: root

    property bool dragging: false
    property url latestFolder: StandardPaths.writableLocation(StandardPaths.DocumentsLocation) + "/neuronify"

    state: "view"

    Settings {
        property alias latestFolder: root.latestFolder
    }

    DownloadManager {
        id: _downloadManager
    }

    Parse {
        id: parse
        debug: true
        serverUrl: "https://parseapi.back4app.com/"
        applicationId: "JffGes20AXUtdN9B6E1RkkHaS7DOxVmxJFSJgLoN"
        restApiKey: "bBKStu7bqeyWFTYFfM5OIes255k9XEz2Voe4fUxS"
    }

    Settings {
        id: settings
        category: "parse"
        property alias sessionToken: parse.sessionToken
    }

    Neuronify {
        id: neuronify
        anchors {
            left: parent.left
            right: parent.right
            top: parent.top
            bottom: parent.bottom
        }
        clip: true
        autoPause: root.state != "view" && root.state != "creation"
    }

    LeftMenu { // TODO rename to topmenu
        id: topMenu

        anchors {
            left: parent.left
            top: parent.top
            right: parent.right
        }

        height: 72
    }

    FileView {
        id: fileView
        anchors {
            left: parent.left
            leftMargin: 72
            top: parent.top
            bottom: parent.bottom
            right: parent.right
        }
        z: 39
    }

    HudShadow {
        id: leftMenuShadow
        anchors.fill: topMenu
        source: topMenu
        z: 38
    }

    Rectangle {
        id: itemMenu
        anchors {
            top: topMenu.bottom
            left: parent.left
            bottom: parent.bottom
        }
        color: "#e3eef9"
        width: 96
        z: 38

        Column {
            anchors {
                top: parent.top
                left: parent.left
                right: parent.right
                topMargin: 16
                leftMargin: 8
                rightMargin: 8
            }

            Repeater {
                id: itemListRepeater
                model: ListModel {
                    ListElement {
                        name: "Excitatory"
                        source: "qrc:/qml/neurons/LeakyNeuron.qml"
                        imageSource: "qrc:/images/neurons/leaky.png"
                        listSource: "qrc:/qml/hud/NeuronList.qml"
                    }
                    ListElement {
                        name: "Inhibitory"
                        listSource: "qrc:/qml/hud/InhibitoryNeuronList.qml"
                        source: "qrc:/qml/neurons/LeakyInhibitoryNeuron.qml"
                        imageSource: "qrc:/images/neurons/leaky_inhibitory.png"
                    }
                    ListElement  {
                        name: "Measurement"
                        listSource: "qrc:/qml/hud/MetersList.qml"
                        source: "qrc:/qml/meters/Voltmeter.qml"
                        imageSource: "qrc:/images/meters/voltmeter.png"
                    }
                    ListElement  {
                        name: "Generators"
                        source: "qrc:/qml/generators/CurrentClamp.qml"
                        imageSource: "qrc:/images/generators/current_clamp.png"
                        listSource: "qrc:/qml/hud/GeneratorsList.qml"
                    }
                    ListElement  {
                        name: "Annotation"
                        source: "qrc:/qml/annotations/Note.qml"
                        listSource: "qrc:/qml/hud/AnnotationsList.qml"
                        imageSource: "qrc:/images/categories/annotation.png"
                    }
                }

                TopCreationItem {
                    id: creationItem

                    parentWhenDragging: root
                    anchors {
                        left: parent.left
                        right: parent.right
                    }

                    name: model.name
                    source: model.source
                    imageSource: model.imageSource
                    selected: Qt.resolvedUrl(itemModelLoader.source) === Qt.resolvedUrl(model.listSource)
                    onClicked: {
                        if(Qt.resolvedUrl(itemModelLoader.source) === Qt.resolvedUrl(model.listSource)) {
                            itemModelLoader.source = ""
                            return
                        }
                        itemModelLoader.source = model.listSource
                    }
                }
            }
        }

        Column {
            anchors {
                left: parent.left
                bottom: parent.bottom
                right: parent.right
                leftMargin: 8
                rightMargin: 8
                bottomMargin: 16
            }

            MaterialButton {
                anchors {
                    left: parent.left
                    right: parent.right
                }
                height: width

                icon.name: "settings_input_component"
                icon.category: "action"
                color: Material.primary
                text: "Properties"
            }
        }
    }

    HudShadow {
        anchors.fill: itemMenu
        source: itemMenu
        z: 29
    }

    Rectangle {
        id: itemSubMenu
        anchors {
            top: topMenu.bottom
            left: itemMenu.right
        }
        width: 240
        height: itemListView.height + 36
        visible: Qt.resolvedUrl(itemModelLoader.source) !== Qt.resolvedUrl("") ? true : false
        color: "#e3eef9"

        Flow {
            id: itemListView
            property int currentIndex: 0
            property alias listSource: itemModelLoader.source
            property int rows: Math.floor(parent.height / 96)
            property int columns: 2
            property real itemHeight: (height - spacing * (rows - 1)) / rows - 1
            property real itemWidth: (width - spacing * (columns - 1)) / columns - 1

            anchors {
                left: parent.left
                right: parent.right
                top: parent.top
                leftMargin: 18
                rightMargin: 18
                topMargin: 18
            }

            spacing: 8

            Loader {
                id: itemModelLoader
//                source: model.listSource
            }

            Repeater {
                model: itemModelLoader.item

                CreationItem {
                    id: creationItem2

                    //                                    width: itemListView.itemWidth
                    width: itemListView.itemWidth

                    parentWhenDragging: root

                    name: model.name
                    description: model.description
                    source: model.source
                    imageSource: model.imageSource

                    onDragActiveChanged: {
                        if(dragActive) {
                            root.dragging = true
                        } else {
                            root.dragging = false
                        }
                        showInfoPanelTimer.stop()
                    }
                }
            }
        }
    }

    HudShadow {
        anchors.fill: itemSubMenu
        source: itemSubMenu
        visible: itemSubMenu.visible
        z: 20
    }

    PropertiesPanel {
        id: propertiesPanel
        anchors {
            left: itemMenu.right
            bottom: parent.bottom
        }

//        width: 320
//        height: 320
        activeObject: neuronify.activeObject
    }

    HudShadow {
        anchors.fill: propertiesPanel
        source: propertiesPanel
        verticalOffset: -1
    }

//    Rectangle {
//        id: itemMenuBackground
//        color: "#e3eef9"
//        anchors {
//            top: leftMenu.bottom
//            left: parent.left
//            bottom: parent.bottom
//        }

//        width: 240

//        MouseArea {
//            anchors.fill: parent
//            hoverEnabled: true
//            onWheel: {
//                // NOTE: necessary to capture wheel events
//            }
//        }

//        QQC1.SplitView {
//            anchors.fill: parent
//            orientation: Qt.Vertical

//            ItemMenu {
//                id: itemMenu
//                anchors {
//                    left: parent.left
//                    right: parent.right
//                }
//                Layout.minimumHeight: 200
//                Layout.fillHeight: true
//            }

//            PropertiesPanel {
//                id: properties
//                anchors {
//                    left: parent.left
//                    right: parent.right
//                }
//                Layout.minimumHeight: 300

//                activeObject: neuronify.activeObject
//                workspace: neuronify.workspace
//            }

//        }
//    }

//    HudShadow {
//        id: itemMenuShadow
//        anchors.fill: itemMenuBackground
//        source: itemMenuBackground
//    }



//    Item {
//        id: infoPanel

//        property var selectedItem

//        anchors {
//            left: itemMenu.right
//            leftMargin: 0
//            top: itemMenu.top
//            topMargin: 12

//            Behavior on topMargin {
//                SmoothedAnimation {
//                    duration: 400
//                    easing.type: Easing.InOutQuad
//                }
//            }
//        }

//        state: "hidden"

//        width: 240
//        height: infoColumn.height + infoColumn.anchors.margins * 2

//        Rectangle {
//            id: infoBackground
//            anchors.fill: parent
//            visible: false
//            color: "#fafafa"
//        }

//        HudShadow {
//            anchors.fill: infoBackground
//            source: infoBackground
//        }

//        Column {
//            id: infoColumn
//            anchors {
//                left: parent.left
//                right: parent.right
//                top: parent.top
//                margins: 16
//                leftMargin: 20
//            }
//            spacing: 8
//            Text {
//                anchors {
//                    left: parent.left
//                    right: parent.right
//                }
//                wrapMode: Text.WrapAtWordBoundaryOrAnywhere
//                font.pixelSize: 18
//                color: "#676767"
//                text: infoPanel.selectedItem ? infoPanel.selectedItem.name : "Nothing selected"
//            }
//            Text {
//                anchors {
//                    left: parent.left
//                    right: parent.right
//                }

//                wrapMode: Text.WrapAtWordBoundaryOrAnywhere
//                font.pixelSize: 14
//                text: infoPanel.selectedItem ? infoPanel.selectedItem.description : "Nothing selected"
//            }
//        }

//        Timer {
//            id: hideInfoPanelTimer
//            interval: 1000
//            onTriggered: {
//                infoPanel.state = "hidden"
//            }
//        }

//        states: [
//            State {
//                name: "hidden"
//                PropertyChanges {
//                    target: infoPanel; anchors.leftMargin: -width
//                }
//            },
//            State {
//                name: "revealed"
//            },
//            State {
//                name: "dragging"
//                extend: "hidden"
//                when: root.dragging
//                onCompleted: infoPanel.state = "hidden"
//                PropertyChanges {
//                    target: infoPanel
//                    opacity: 0.0
//                }
//            }
//        ]

//        transitions: [
//            Transition {
//                NumberAnimation {
//                    properties: "anchors.leftMargin"
//                    duration: 800
//                    easing.type: Easing.InOutQuad
//                }
//                NumberAnimation {
//                    properties: "opacity"
//                    duration: 200
//                }
//            }
//        ]
//    }

    states: [
        State {
            name: "view"
            PropertyChanges { target: fileView; state: "hidden" }
//            PropertyChanges { target: infoPanel; state: "hidden" }
        },
        State {
            name: "creation"
            extend: "view"
            PropertyChanges { target: topMenu; state: "small" }
            PropertyChanges { target: itemMenu; state: "" }
        },
        State {
            name: "welcome"
            extend: "view"
            PropertyChanges { target: fileView; state: "" }
            PropertyChanges { target: leftMenuShadow; opacity: 0.0 }
            PropertyChanges { target: topMenu; state: "small" }
        },
        State {
            name: "projects"
            extend: "view"
        },
        State {
            name: "help"
            extend: "view"
        }

    ]

    transitions: [
        Transition {
            animations: [
                animateCreation,
            ]
        },
        Transition {
            to: "community"
            animations: [
                animateCreation,
                animateCommunityTextIn
            ]
        },
        Transition {
            from: "community"
            animations: [
                animateCreation,
                animateCommunityTextOut
            ]
        }
    ]

    ParallelAnimation {
        id: animateCreation
        NumberAnimation {
            properties: "anchors.leftMargin"
            duration: 400
            easing.type: Easing.InOutQuad
        }
        ColorAnimation {
            properties: "color"
            duration: 400
            easing.type: Easing.InOutQuad
        }
    }

    SequentialAnimation {
        id: animateCommunityTextIn
        PauseAnimation {
            duration: 400
        }
    }
    SequentialAnimation {
        id: animateCommunityTextOut
        PauseAnimation {
            duration: 200
        }
    }
}
