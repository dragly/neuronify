import QtQuick 2.6
import QtQuick.Controls 1.4

import "../style"

Item {
    id: root

    property var workspace
    property Item activeObject: null
    property bool revealed: false

    anchors.fill: parent

    onActiveObjectChanged: {
        stackView.clear()
        if(activeObject && activeObject.controls) {
            stackView.push(activeObject.controls)
        }
    }

    Rectangle {
        id: background
        anchors {
            right: parent.right
            top: parent.top
            rightMargin: -width
            bottom: parent.bottom
        }

        color: "#f7fbff"
        width: Style.device === "phone" ? parent.width * 0.5 : parent.width * 0.25
//        width: parent.width * 0.5

        border.color: "#9ecae1"
        border.width: 1.0

        MouseArea {
            anchors.fill: parent
            propagateComposedEvents: false
            onClicked: {
                mouse.accepted = true
            }
            onPressed: {
                mouse.accepted = true
            }
            onReleased: {
                mouse.accepted = true
            }
            onWheel: {
                wheel.accepted = true
            }
        }

        Item {
            anchors.fill: parent
            clip: true
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
                    source: "qrc:/images/back.png"
                    states: [
                        State {
                            when: stackView.depth > 1 ? 0 : -width
                            PropertyChanges {
                                target: backButton
                                anchors.leftMargin: -backButton.width
                            }
                        },
                        State {
                            when: !stackView.currentItem || !stackView.currentItem.title || stackView.currentItem.title === ""
                            PropertyChanges {
                                target: backButton
                                anchors.topMargin: -backButton.height
                            }
                        }

                    ]
                    transitions: [
                        Transition {
                            NumberAnimation {
                                properties: "anchors.leftMargin, anchors.topMargin"
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

                Rectangle {
                    anchors {
                        left: parent.left
                        right: parent.right
                        bottom: parent.bottom
                    }
                    height: Style.border.width * 2.0
                    color: Style.border.color
                }

                MouseArea {
                    anchors {
                        fill: parent
                    }

                    onClicked: {
                        stackView.pop()
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
            }
        }

        states: State {
            when: root.revealed
            PropertyChanges {
                target: background
                anchors.rightMargin: 0.0
            }
        }

        transitions: Transition {
            NumberAnimation {
                target: background
                property: "anchors.rightMargin"
                duration: 400
                easing.type: Easing.InOutCubic
            }
        }
    }
}
