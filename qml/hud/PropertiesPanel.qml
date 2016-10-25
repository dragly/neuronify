import QtQuick 2.6
import QtQuick.Controls 1.4
import QtQuick.Dialogs 1.2

import "../controls"
import "../hud"
import "../style"

Item {
    id: root

    signal resetDynamics
    signal resetProperties
    signal saveToOpened

    readonly property real offset: background.anchors.rightMargin + background.width

    property var workspace
    property Item activeObject: null
    property bool revealed: false
    property bool advanced
    property bool snappingEnabled

    function open() {
        revealed = true
    }

    function close() {
        revealed = false
    }

    onRevealedChanged: {
        if (revealed) {
            focus = true
        }
        else {
            focus = false
        }
    }

    anchors.fill: parent

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

    Rectangle {
        id: background
        anchors {
            right: parent.right
            top: parent.top
            rightMargin: -width
            bottom: parent.bottom
        }

        color: "#f7fbff"
        width: {
            if(Style.device === "phone") {
                if(parent.width > parent.height) {
                    return parent.width * 0.5;
                } else {
                    return parent.width * 0.85;
                }
            } else {
                return parent.width * 0.4;
            }
        }

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
                            root.revealed = false;
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

    Keys.onPressed: {
        console.log("caught button press PROPERTIES")
        if(event.key === Qt.Key_Back || event.key === Qt.Key_Escape) {
            if(stackView.depth > 1){
                stackView.pop();
            } else {
                revealed = false
            }
        }
    }
}
