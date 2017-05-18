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

    signal resetDynamics
    signal resetProperties
    signal saveToOpened

    readonly property real offset: background.anchors.rightMargin + background.width

    property var workspace
    property Item activeObject: null
    property bool revealed: false
    property bool advanced
    property bool snappingEnabled

    width: loader.width
    height: loader.height

    Material.theme: Material.Light

    Rectangle {
        id: background
        anchors.fill: parent

        color: "#e3eef9"
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

    HudShadow {
        anchors.fill: background
        source: background
    }

    Loader {
        id: loader
        clip: true
        sourceComponent: activeObject && activeObject.controls ? activeObject.controls : simulationComponent
    }

    Component {
        id: simulationComponent
        PropertiesContainer {
            PropertiesPage {
                id: simulatonPage
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
                Button {
                    visible: root.advanced
                    text: "Save to opened"
                    onClicked: {
                        root.saveToOpened();
                    }
                }
            }
        }
    }

    Keys.onPressed: {
        if(event.key === Qt.Key_Back || event.key === Qt.Key_Escape) {
            if(stackView.depth > 1){
                stackView.pop();
            } else {
                revealed = false
            }
            event.accepted = true
        }
    }
}
