import QtQuick 2.5
import QtQuick.Controls 2.0
import QtQuick.Dialogs 1.0
import QtQuick.Layouts 1.3
import QtQuick.Particles 2.0
import QtQuick.Window 2.1

import QtCharts 2.1
import QtMultimedia 5.5
import Qt.labs.settings 1.0

import Neuronify 1.0
import CuteVersioning 1.0
import QtGraphicalEffects 1.0

import "qrc:/qml/hud"
import "qrc:/qml/menus/mainmenu"
import "qrc:/qml/style"
import "qrc:/qml/io"
import "qrc:/qml/tools"
import "qrc:/qml/controls"

Item {
    id: root

    property bool dragging: false

    state: "community"

    Neuronify {
        id: neuronify
        anchors {
            left: parent.left
            right: parent.right
            top: parent.top
            bottom: parent.bottom
        }
        clip: true
    }

    Rectangle {
        id: leftMenu

        property real textOpacity: 1.0

        anchors {
            left: parent.left
            top: parent.top
            bottom: parent.bottom
            leftMargin: 0
        }

        width: 128

        color: "#6BAED6"
        z: 40

        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
        }

        Image {
            id: logo
            anchors {
                left: parent.left
                right: parent.right
                top: parent.top
                margins: parent.width * 0.2
                topMargin: 48
            }
            fillMode: Image.PreserveAspectFit
            height: width
            source: "qrc:/images/logo/logo-no-background.png"
            mipmap: true
        }

        Text {
            id: logoText
            color: "white"
            font.pixelSize: 24
            font.family: Style.font.family
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WrapAtWordBoundaryOrAnywhere
            text: "Neuronify\n" + Version.latestTag
        }

        ShaderEffectSource {
            id: logoTextCopy
            anchors {
                left: parent.left
                right: parent.right
                top: logo.bottom
                //                top: parent.top
                margins: 8
            }
            height: width * logoText.height / logoText.width
            hideSource: true
            sourceItem: logoText
            smooth: true
            antialiasing: true
        }

        Column {
            anchors {
                left: parent.left
                right: parent.right
                top: logoTextCopy.bottom
                topMargin: 24
                bottom: parent.bottom
            }
            spacing: 24
            Repeater {
                model: ListModel {
                    ListElement {
                        state: "projects"
                        name: "Welcome"
                    }
                    ListElement {
                        state: "view"
                        name: "View"
                    }
                    ListElement {
                        state: "creation"
                        name: "Create"
                    }
                    ListElement {
                        state: "save"
                        name: "Save"
                    }
                    ListElement {
                        state: "community"
                        name: "Community"
                    }
                    ListElement {
                        state: "help"
                        name: "Help"
                    }
                }
                MouseArea {
                    anchors {
                        left: parent.left
                        right: parent.right
                    }
                    height: menuItemColumn.height
                    onClicked: {
                        root.state = model.state
                    }
                    Column {
                        id: menuItemColumn
                        anchors {
                            left: parent.left
                            right: parent.right
                        }

                        spacing: 8
                        Rectangle {
                            anchors.horizontalCenter: parent.horizontalCenter
                            width: parent.width / 2
                            height: width
                            radius: width / 4
                            color: root.state == model.state ? "white" : "transparent"
                            border.width: parent.width * 0.04
                            border.color: "white"
                        }
                        Text {
                            anchors {
                                left: parent.left
                                right: parent.right
                            }
                            horizontalAlignment: Text.AlignHCenter
                            color: "white"
                            opacity: leftMenu.textOpacity
                            font.pixelSize: 12
                            text: model.name
                        }
                    }
                }
            }
        }

        onStateChanged: console.log("Left menu state", state)

        states: [
            State {
                name: "small"
                PropertyChanges { target: leftMenu; width: 72 }
                PropertyChanges { target: logoTextCopy; opacity: 0.0 }
                PropertyChanges { target: leftMenu; textOpacity: 0.0 }
            }
        ]

        transitions: [
            Transition {
                NumberAnimation {
                    target: leftMenu
                    properties: "width,textOpacity"
                    duration: 400
                    easing.type: Easing.InOutQuad
                }
                NumberAnimation {
                    target: leftMenuShadow
                    properties: "opacity"
                    duration: 0
                }
            }
        ]
    }

    Rectangle {
        id: community
        anchors {
            left: leftMenu.right
            top: parent.top
            bottom: parent.bottom
        }
        radius: height / 2
        width: parent.width
        color: leftMenu.color
        z: 39
        state: "hidden"

        Loader {
            anchors.fill: parent
            source: "store/Store.qml"
        }

        states: [
            State {
                name: "hidden"
                PropertyChanges {
                    target: community
                    anchors.leftMargin: -width
                }
            }
        ]

        transitions: [
            Transition {
                NumberAnimation {
                    duration: 600
                    properties: "anchors.leftMargin"
                    easing.type: Easing.OutQuad
                }
            }
        ]
    }

    HudShadow {
        id: leftMenuShadow
        anchors.fill: leftMenu
        source: leftMenu
        z: 38
    }

    Item {
        id: itemMenu

        anchors {
            left: leftMenu.right
            top: parent.top
            topMargin: 64
            bottom: parent.bottom
            bottomMargin: 64
            //            bottomMargin: 120
        }

        width: 280 + 32
        height: itemColumn.height
        z: 20

        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            onWheel: {
                // NOTE: necessary to capture wheel events
            }
        }

        ListModel {
            id: categories
            ListElement {
                listSource: "qrc:/qml/hud/NeuronList.qml"
                imageSource: "qrc:/images/categories/excitatory.png"
                text: "Excitatory neurons"
            }
            ListElement  {
                listSource: "qrc:/qml/hud/InhibitoryNeuronList.qml"
                imageSource: "qrc:/images/categories/inhibitory.png"
                text: "Inhibitory neurons"
            }

            ListElement  {
                listSource: "qrc:/qml/hud/MetersList.qml"
                imageSource: "qrc:/images/categories/meters.png"
                text: "Measurement devices"
            }

            ListElement  {
                listSource: "qrc:/qml/hud/GeneratorsList.qml"
                imageSource: "qrc:/images/categories/generators.png"
                text: "Generators"
            }
            ListElement  {
                listSource: "qrc:/qml/hud/AnnotationsList.qml"
                imageSource: "qrc:/images/categories/annotation.png"
                text: "Annotation"
            }
        }

        Rectangle {
            id: itemMenuBackground
            color: "#e3eef9"
            anchors {
                fill: itemFlickable
                topMargin: -16
                bottomMargin: -16
            }
        }

        HudShadow {
            id: itemMenuShadow
            anchors.fill: itemMenuBackground
            source: itemMenuBackground
        }

        Flickable {
            id: itemFlickable
            anchors {
                left: parent.left
                right: parent.right
            }

            height: Math.min(parent.height, itemColumn.height)
            clip: true

//            ScrollIndicator.vertical: ScrollIndicator {}
            ScrollBar.vertical: ScrollBar {}
            contentHeight: itemColumn.height
//            interactive: false

            Column {
                id: itemColumn
                property int currentIndex: -1

                anchors {
                    left: parent.left
                    right: parent.right
                }

                Component.onCompleted: {
                    currentIndex = 0
                }

                Repeater {
                    model: categories
                    Column {
                        anchors {
                            left: parent.left
                            right: parent.right
                        }
                        spacing: 12
                        Text {
                            anchors {
                                left: parent.left
                                right: parent.right
                                margins: 16
                            }
                            font.pixelSize: 18
                            font.family: Style.font.family
                            color: Style.creation.text.color
                            wrapMode: Text.WrapAtWordBoundaryOrAnywhere
                            text: model.text
                        }

                        Flow {
                            id: itemListView
                            property int currentIndex: 0
                            property alias listSource: itemModelLoader.source
                            property int rows: Math.floor(parent.height / 96)
                            property int columns: 3
                            property real itemHeight: (height - spacing * (rows - 1)) / rows
                            property real itemWidth: (width - spacing * (columns - 1)) / columns

                            anchors {
                                left: parent.left
                                right: parent.right
                                leftMargin: 24
                                rightMargin: 48
                            }

                            spacing: 8

                            Loader {
                                id: itemModelLoader
                                source: model.listSource
                            }

                            Repeater {
                                id: itemListRepeater

                                model: itemModelLoader.item

                                CreationItem {
                                    id: creationItem

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

                                    MouseArea {
                                        id: hoverArea
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        acceptedButtons: Qt.NoButton
                                        propagateComposedEvents: true
                                        onEntered: {
                                            infoPanel.selectedItem = creationItem
                                            hideInfoPanelTimer.stop()
                                            showInfoPanelTimer.restart()
                                        }
                                        onExited: {
                                            hideInfoPanelTimer.restart()
                                            showInfoPanelTimer.stop()
                                        }
                                    }

                                    Timer {
                                        id: showInfoPanelTimer
                                        interval: 400
                                        onTriggered: {
                                            if(hoverArea.containsMouse) {
                                                infoPanel.state = "revealed"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        states: [
            State {
                name: "dragging"
                when: root.dragging
                PropertyChanges {
                    target: itemMenu
                    opacity: 0.0
                }
            },
            State {
                name: "hidden"
                PropertyChanges { target: itemMenu; anchors.leftMargin: -itemMenu.width }
            }
        ]

        transitions: [
            Transition {
                NumberAnimation {
                    properties: "opacity"
                    duration: 200
                }
                NumberAnimation {
                    properties: "anchors.leftMargin"
                    duration: 400
                    easing.type: Easing.InOutQuad
                }
            }
        ]
    }

    Item {
        id: saveMenu

        anchors {
            left: leftMenu.right
            top: parent.top
            topMargin: 64
            bottom: parent.bottom
            bottomMargin: 64
            //            bottomMargin: 120
        }

        width: 280 + 32
        height: itemColumn.height
        z: 20

        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            onWheel: {
                // NOTE: necessary to capture wheel events
            }
        }

        Rectangle {
            id: saveMenuBackground
            color: "#e3eef9"
            anchors {
                fill: parent
                topMargin: -16
                bottomMargin: -16
            }
        }

        HudShadow {
            id: saveMenuShadow
            anchors.fill: saveMenuBackground
            source: saveMenuBackground
        }

        states: [
            State {
                name: "hidden"
                PropertyChanges { target: saveMenu; anchors.leftMargin: -saveMenu.width }
            }
        ]

        transitions: [
            Transition {
                NumberAnimation {
                    properties: "opacity"
                    duration: 200
                }
                NumberAnimation {
                    properties: "anchors.leftMargin"
                    duration: 400
                    easing.type: Easing.InOutQuad
                }
            }
        ]
    }

    Item {
        id: infoPanel

        property var selectedItem

        anchors {
            left: itemMenu.right
            leftMargin: 0
            top: itemMenu.top
            topMargin: {
//                itemFlickable.contentY // dummy to ensure updates on scroll
//                infoPanel.selectedItem ? itemMenu.mapFromItem(infoPanel.selectedItem, 0, 0).y : 0
                return 12
            }

            Behavior on topMargin {
                SmoothedAnimation {
                    duration: 400
                    easing.type: Easing.InOutQuad
                }
            }
        }

        state: "hidden"

        width: 240
        height: infoColumn.height + infoColumn.anchors.margins * 2

        Rectangle {
            id: infoBackground
            anchors.fill: parent
            visible: false
            color: "#fafafa"
        }

        HudShadow {
            anchors.fill: infoBackground
            source: infoBackground
        }

        Column {
            id: infoColumn
            anchors {
                left: parent.left
                right: parent.right
                top: parent.top
                margins: 16
                leftMargin: 20
            }
            spacing: 8
            Text {
                anchors {
                    left: parent.left
                    right: parent.right
                }
                wrapMode: Text.WrapAtWordBoundaryOrAnywhere
                font.pixelSize: 18
                color: "#676767"
                text: infoPanel.selectedItem ? infoPanel.selectedItem.name : "Nothing selected"
            }
            Text {
                anchors {
                    left: parent.left
                    right: parent.right
                }

                wrapMode: Text.WrapAtWordBoundaryOrAnywhere
                font.pixelSize: 14
                text: infoPanel.selectedItem ? infoPanel.selectedItem.description : "Nothing selected"
            }
        }

        Timer {
            id: hideInfoPanelTimer
            interval: 1000
            onTriggered: {
                infoPanel.state = "hidden"
            }
        }

        states: [
            State {
                name: "hidden"
                PropertyChanges {
                    target: infoPanel; anchors.leftMargin: -width
                }
            },
            State {
                name: "revealed"
            },
            State {
                name: "dragging"
                extend: "hidden"
                when: root.dragging
                onCompleted: infoPanel.state = "hidden"
                PropertyChanges {
                    target: infoPanel
                    opacity: 0.0
                }
            }
        ]

        transitions: [
            Transition {
                NumberAnimation {
                    properties: "anchors.leftMargin"
                    duration: 800
                    easing.type: Easing.InOutQuad
                }
                NumberAnimation {
                    properties: "opacity"
                    duration: 200
                }
            }
        ]
    }

    states: [
        State {
            name: "view"
            PropertyChanges { target: community; state: "hidden" }
            PropertyChanges { target: infoPanel; state: "hidden" }
            PropertyChanges { target: itemMenu; state: "hidden" }
            PropertyChanges { target: saveMenu; state: "hidden" }
        },
        State {
            name: "creation"
            extend: "view"
            PropertyChanges { target: leftMenu; state: "small" }
            PropertyChanges { target: itemMenu; state: "" }
        },
        State {
            name: "save"
            extend: "view"
            PropertyChanges { target: leftMenu; state: "small" }
            PropertyChanges { target: saveMenu; state: "" }
        },
        State {
            name: "community"
            extend: "view"
            PropertyChanges { target: community; state: "" }
            PropertyChanges { target: leftMenuShadow; opacity: 0.0 }
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
                animateLeftMenu,
            ]
        },
        Transition {
            to: "community"
            animations: [
                animateLeftMenu,
                animateCreation,
                animateCommunityTextIn
            ]
        },
        Transition {
            from: "community"
            animations: [
                animateLeftMenu,
                animateCreation,
                animateCommunityTextOut
            ]
        }
    ]

    ParallelAnimation {
        id: animateLeftMenu
        NumberAnimation {
            target: logoTextCopy
            property: "opacity"
            duration: 400
            easing.type: Easing.InOutQuad
        }
    }

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
