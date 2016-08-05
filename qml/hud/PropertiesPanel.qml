import QtQuick 2.6
import QtQuick.Controls 1.4
import QtQuick.Controls 2.0 as QQC2
import QtQuick.Dialogs 1.2

import "../controls"
import "../hud"
import "../style"

QQC2.Drawer {
    id: root

    signal resetDynamics
    signal resetProperties
    signal saveToOpened

    property var workspace
    property Item activeObject: null
    property bool advanced
    property bool snappingEnabled

    width: Math.min(parent.width, parent.height) * 0.85
    height: parent.height
    edge: Qt.RightEdge

    Component.onCompleted: {
        stackView.push(simulationComponent);
    }

    onActiveObjectChanged: {
        stackView.clear()
        if(activeObject && activeObject.controls) {
            stackView.push(activeObject.controls)
        } else {
            stackView.push(simulationComponent);
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
                source: "qrc:/images/tools/back.png"
                states: [
                    State {
                        when: stackView.depth > 1 ? 0 : -width
                        PropertyChanges {
                            target: backButton
                            rotation: 180
                        }
                    }

                ]
                transitions: [
                    Transition {
                        NumberAnimation {
                            properties: "rotation"
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
                    if(stackView.depth > 1) {
                        stackView.pop();
                    } else {
                        root.close()
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
        }
    }

    Component {
        id: simulationComponent
        PropertiesPage {
            id: simulatonPage
            property bool advanced: root.advanced
            title: "Simulation"
            Button {
                text: "Reset all dynamics"
                onClicked: {
                    root.resetDynamics();
                }
            }
            Button {
                text: "Reset all properties"
                onClicked: {
                    resetDialog.open();
                }
                MessageDialog {
                    id: resetDialog
                    text: "This will reset all properties for all items. Are you sure?"
                    standardButtons: StandardButton.Ok | StandardButton.Cancel
                    onAccepted: {
                        root.resetProperties();
                    }
                }
            }
            CheckBox {
                id: snapCheckBox
                text: "Snap to grid"
                checked: root.snappingEnabled
                Binding {
                    target: root
                    property: "snappingEnabled"
                    value: snapCheckBox.checked
                }
                Binding {
                    target: snapCheckBox
                    property: "checked"
                    value: root.snappingEnabled
                }
            }
            CheckBox {
                id: advancedCheckBox
                text: "Show advanced features"
                Binding {
                    target: advancedCheckBox
                    property: "checked"
                    value: root.advanced
                }
                Binding {
                    target: root
                    property: "advanced"
                    value: advancedCheckBox.checked
                }
            }
            Button {
                visible: simulatonPage.advanced
                text: "Save to opened"
                onClicked: {
                    root.saveToOpened();
                }
            }
        }
    }
}
