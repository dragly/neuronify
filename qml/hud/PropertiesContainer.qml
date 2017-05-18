import QtQuick 2.6
import QtQuick.Controls 1.4
import QtQuick.Controls 2.1
import QtQuick.Controls.Material 2.1
import QtQuick.Dialogs 1.2

import "../controls"
import "../hud"
import "../style"

Item {
    id: root
    default property var initialItem

    Item {
        id: header
        
        anchors {
            left: parent.left
            right: parent.right
        }
        
        height: Style.control.fontMetrics.height * 2.2
        
        Image {
            id: backButton
            anchors {
                left: parent.left
                top: parent.top
                bottom: parent.bottom
                topMargin: parent.height * 0.2
                bottomMargin: parent.height * 0.2
                leftMargin: Style.spacing
            }
            width: height
            source: "qrc:/images/tools/back.png"
            states: [
                State {
                    when: stackView.depth < 2
                    PropertyChanges {
                        target: backButton
                        rotation: 180
                        opacity: 0
                    }
                }
                
            ]
            transitions: [
                Transition {
                    NumberAnimation {
                        properties: "rotation,opacity"
                        duration: 400
                        easing.type: Easing.InOutQuad
                    }
                }
            ]
        }
        
        Text {
            id: titleText
            text: stackView.currentItem && stackView.currentItem.title ? stackView.currentItem.title : ""
            anchors {
                left: backButton.right
                right: parent.right
                verticalCenter: backButton.verticalCenter
                margins: Style.spacing
            }
            font: Style.control.heading.font
        }
        
        MouseArea {
            anchors {
                fill: parent
            }
            
            onClicked: {
                if(stackView.depth > 1) {
                    stackView.pop()
                } else {
                }
            }
        }
    }
    
    StackView {
        id: stackView
        anchors {
            left: parent.left
            right: parent.right
            bottom: parent.bottom
            top: header.bottom
            topMargin: 4
            leftMargin: Style.spacing
            rightMargin: Style.spacing
        }
        clip: true
        initialItem: root.initialItem
    }
}
