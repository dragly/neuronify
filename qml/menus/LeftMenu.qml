import QtQuick 2.5
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
import "qrc:/qml/menus/filemenu"
import "qrc:/qml/menus/mainmenu"
import "qrc:/qml/tools"
import "qrc:/qml/store"
import "qrc:/qml/style"
import "qrc:/qml/ui"

Rectangle {
    id: leftMenu
    
    property real textOpacity: 1.0
    
    Material.theme: Material.Dark
    
    //        color: "#1782C2"
    //        color: Material.color(Material.Cyan)
    color: Material.primary
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
        id: menuColumn
        anchors {
            left: parent.left
            right: parent.right
            top: logoTextCopy.bottom
            topMargin: 48
        }
        spacing: 24
        Repeater {
            model: ListModel {
                ListElement {
                    state: "welcome"
                    name: "Simulation"
                    category: "action"
                    icon: "view_list"
                }
                ListElement {
                    state: "view"
                    name: "View"
                    category: "image"
                    icon: "remove_red_eye"
                }
                ListElement {
                    state: "creation"
                    name: "Create"
                    category: "content"
                    icon: "create"
                }
                ListElement {
                    state: "help"
                    name: "Help"
                    category: "action"
                    icon: "help_outline"
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
                    MaterialIcon {
                        anchors.horizontalCenter: parent.horizontalCenter
                        width: parent.width * 0.4
                        height: width
                        //                            radius: width / 4
                        color: root.state == model.state ? "white" : "#aaffffff"
                        //                            border.width: parent.width * 0.04
                        //                            border.color: "white"
                        name: model.icon
                        category: model.category
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
    
    states: [
        State {
            name: "small"
            PropertyChanges { target: leftMenu; width: 72 }
            PropertyChanges { target: logoTextCopy; opacity: 0.0 }
            PropertyChanges { target: leftMenu; textOpacity: 0.0 }
        },
        State {
            name: "hidden"
            //                AnchorChanges {
            //                    target: leftMenu
            //                    anchors {
            //                        left: undefined
            //                        right: parent.left
            //                    }
            //                }
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
            AnchorAnimation {
                duration: 0
            }
            NumberAnimation {
                target: logoTextCopy
                property: "opacity"
                duration: 400
                easing.type: Easing.InOutQuad
            }
        },
        Transition {
            from: "hidden"
            to: ""
            AnchorAnimation {
                duration: 400
                easing.type: Easing.InOutQuad
            }
        }
    ]
}
