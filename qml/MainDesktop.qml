import QtQuick 2.5
import QtQuick.Controls 2.1
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

    state: "creation"

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
        anchors {
            left: parent.left
            top: parent.top
            bottom: parent.bottom
            leftMargin: 0
        }

        width: 128

        color: "#6BAED6"
        z: 40

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
                verticalCenter: parent.verticalCenter
            }
            spacing: 24
            Rectangle {
                anchors.horizontalCenter: parent.horizontalCenter
                width: parent.width / 2
                height: width
                radius: width / 4
                color: "transparent"
                border.width: parent.width * 0.04
                border.color: "white"
                MouseArea {
                    anchors.fill: parent
                    onClicked: {
                        root.state = ""
                    }
                }
            }

            Rectangle {
                anchors.horizontalCenter: parent.horizontalCenter
                width: parent.width / 2
                height: width
                radius: width / 4
                color: "transparent"
                border.width: parent.width * 0.04
                border.color: "white"
                MouseArea {
                    anchors.fill: parent
                    onClicked: {
                        root.state = "creation"
                    }
                }
            }

            Rectangle {
                anchors.horizontalCenter: parent.horizontalCenter
                width: parent.width / 2
                height: width
                radius: width / 4
                color: "transparent"
                border.width: parent.width * 0.04
                border.color: "white"
                MouseArea {
                    anchors.fill: parent
                    onClicked: {
                        root.state = "community"
                    }
                }
            }
        }
    }

    Rectangle {
        id: communityBackground
        anchors {
            left: parent.left
            top: parent.top
            bottom: parent.bottom
        }
        radius: height / 2
        width: 0
        color: leftMenu.color
        z: 39
    }

    Text {
        id: communityText
        anchors {
            top: parent.top
            left : leftMenu.right
            topMargin: 48
            leftMargin: 240
        }
        text: "Neuronify community"
        font.pixelSize: 48
        color: "white"
        z: 40
        opacity: 0.0
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
            left: subItemMenu.right
            //            left: parent.left
            leftMargin: -width
            top: parent.top
            topMargin: 64
//            bottom: parent.bottom
            //            bottomMargin: 120
        }

        width: 240 + 32
        height: categoriesListView.height
        z: 5

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
            id: rect
            color: "#e3eef9"
            anchors {
                fill: parent
                topMargin: -16
                bottomMargin: -16
            }
        }

        HudShadow {
            anchors.fill: rect
            source: rect
        }

        Column {
            id: categoriesListView
            property int currentIndex: -1

            anchors {
                left: parent.left
                right: parent.right
            }

            Component.onCompleted: {
                currentIndex = 0
            }

            onCurrentIndexChanged: {
                stackView.replace(viewComponent, {listSource: categories.get(currentIndex).listSource})
            }

            Repeater {
                model: categories
                Item {
                    anchors {
                        left: parent.left
                        right: parent.right
                    }
                    height: 72

                    Rectangle {
                        anchors {
                            fill: parent
                        }
                        color: Qt.hsla(0.0, 0.0, 1.0, 0.6)
                        smooth: true
                        antialiasing: true
                        visible: index === categoriesListView.currentIndex
                    }
                    Text {
                        anchors {
                            left: parent.left
                            right: parent.right
                            verticalCenter: parent.verticalCenter
                            margins: 16
                        }
                        font.pixelSize: 18
                        font.family: Style.font.family
                        color: "#2d76a2"
                        wrapMode: Text.WrapAtWordBoundaryOrAnywhere
                        text: model.text
                    }

//                        Image {
//                            anchors.centerIn: parent
//                            width: parent.width * 0.6
//                            height: width

//                            asynchronous: true
//                            source: imageSource
//                            antialiasing: true
//                            smooth: true
//                        }
                    MouseArea {
                        anchors.fill: parent
                        onClicked: {
                            categoriesListView.currentIndex = index
                        }
                    }
                }
            }
        }

        states: [
            State {
                name: "dragging"
                PropertyChanges {
                    target: itemMenu
                    opacity: 0.0
                }
            }
        ]

        transitions: [
            Transition {
                NumberAnimation {
                    properties: "opacity"
                    duration: 200
                }
            }
        ]
    }

    Item {
        id: subItemMenu
        anchors {
            left: leftMenu.right
            leftMargin: -width
            top: parent.top
            bottom: parent.bottom
            //            topMargin: 64
        }

        width: 160
        height: 480
        z: 10
        state: itemMenu.state

        Rectangle {
            id: subItemBackground
            anchors.fill: parent
            color: "#fefefe"
            visible: false
        }

        HudShadow {
            anchors.fill: subItemBackground
            source: subItemBackground
        }

        Component {
            id: viewComponent
            Column {
                id: itemListView
                property int currentIndex: 0
                property alias listSource: itemModelLoader.source
                Loader {
                    id: itemModelLoader
                }
                Repeater {
                    id: itemListRepeater

                    model: itemModelLoader.item

                    Item {
                        property var item: creationItem
                        anchors {
                            left: parent.left
                            right: parent.right
                        }
                        height: width * 0.6

                        CreationItem {
                            id: creationItem
                            anchors.centerIn: parent

                            width: parent.width * 0.5
                            height: width
                            parentWhenDragging: root

                            name: model.name
                            description: model.description
                            source: model.source
                            imageSource: model.imageSource

                            onDragActiveChanged: {
                                if(drag.active) {
                                    subItemMenu.state = "dragging"
                                    itemMenu.state = "dragging"
                                } else {
                                    subItemMenu.state = ""
                                    itemMenu.state = ""
                                }
                            }

                            onPressed: {
                                itemListView.currentIndex = index
                            }

                            onDropped: {
//                                droppedEntity(fileUrl, properties, controlParent)
                            }
                        }
                    }
                }
            }
        }

        StackView {
            id: stackView
            anchors {
                fill: parent
            }
            clip: true

            replaceEnter: Transition {
                NumberAnimation {
                    properties: "x"
                    from: stackView.width / 2
                    to: 0
                    duration: 400
                    easing.type: Easing.InOutQuad
                }
                NumberAnimation {
                    properties: "opacity"
                    from: 0
                    to: 1
                    duration: 600
                    easing.type: Easing.InOutQuad
                }
            }

            replaceExit: Transition {
                NumberAnimation {
                    properties: "x"
                    from: 0
                    to: -stackView.width / 2
                    duration: 400
                    easing.type: Easing.InOutQuad
                }
                NumberAnimation {
                    properties: "opacity"
                    from: 1
                    to: 0
                    duration: 200
                    easing.type: Easing.InOutQuad
                }
            }
        }

        states: [
            State {
                name: "dragging"
                PropertyChanges {
                    target: subItemMenu
                    opacity: 0.0
                }
            }
        ]

        transitions: [
            Transition {
                NumberAnimation {
                    properties: "opacity"
                    duration: 200
                }
            }
        ]
    }

    states: [
        State {
            name: "creation"
            PropertyChanges { target: itemMenu; anchors.leftMargin: 0 }
            PropertyChanges { target: subItemMenu; anchors.leftMargin: 0}
            PropertyChanges { target: leftMenu; width: 72 }
            PropertyChanges { target: logoTextCopy; opacity: 0.0 }
        },
        State {
            name: "community"
            PropertyChanges { target: leftMenuShadow; opacity: 0.0 }
            PropertyChanges { target: communityBackground; width: parent.width }
            PropertyChanges { target: communityBackground; radius: 0 }
            PropertyChanges {
                target: communityText
                anchors.leftMargin: 48
                opacity: 1.0
            }
        }
    ]

    transitions: [
        Transition {
            animations: [
                animateLeftMenu,
                animateCreation
            ]
        },
        Transition {
            to: "community"
            animations: [
                animateLeftMenu,
                animateCreation,
                animateCommunityBackground,
                animateCommunityTextIn
            ]
        },
        Transition {
            from: "community"
            animations: [
                animateLeftMenu,
                animateCreation,
                animateCommunityBackground,
                animateCommunityTextOut
            ]
        }
    ]

    ParallelAnimation {
        id: animateLeftMenu
        NumberAnimation {
            target: leftMenu
            property: "width"
            duration: 400
            easing.type: Easing.InOutQuad
        }
        NumberAnimation {
            target: logoTextCopy
            property: "opacity"
            duration: 400
            easing.type: Easing.InOutQuad
        }
        NumberAnimation {
            target: leftMenuShadow
            properties: "opacity"
            duration: 0
        }
    }

    ParallelAnimation {
        id: animateCreation
        NumberAnimation {
            target: itemMenu
            properties: "anchors.leftMargin"
            duration: 600
            easing.type: Easing.InOutQuad
        }
        NumberAnimation {
            target: subItemMenu
            properties: "anchors.leftMargin"
            duration: 800
            easing.type: Easing.InOutQuad
        }
    }

    SequentialAnimation {
        id: animateCommunityTextIn
        PauseAnimation {
            duration: 400
        }
        NumberAnimation {
            target: communityText
            properties: "opacity,anchors.leftMargin"
            duration: 800
            easing.type: Easing.OutQuad
        }
    }
    SequentialAnimation {
        id: animateCommunityTextOut
        NumberAnimation {
            target: communityText
            properties: "opacity"
            duration: 200
            easing.type: Easing.InOutQuad
        }
        PauseAnimation {
            duration: 200
        }
        NumberAnimation {
            target: communityText
            properties: "anchors.leftMargin"
            duration: 0
        }
    }
    NumberAnimation {
        id: animateCommunityBackground
        target: communityBackground
        properties: "width"
        duration: 600
        easing.type: Easing.InOutQuad
    }
}
