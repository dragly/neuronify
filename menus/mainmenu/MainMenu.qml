import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import QtGraphicalEffects 1.0
import "../../style"

Item {
    id: mainMenuRoot
    property bool revealed: true
    property var blurSource: null
    signal loadSimulation(var simulation)
    signal saveSimulationRequested
    signal loadSimulationRequested

    width: 100
    height: 62

    function hide() {
        revealed = false
    }

    MouseArea {
        enabled: mainMenuRoot.revealed
        anchors.fill: parent
    }

    Item {
        id: background
        anchors.fill: parent

        FastBlur {
            anchors.fill: parent
            source: blurSource
            radius: 64
        }

        Rectangle {
            anchors.fill: parent
            color: Qt.rgba(1.0, 1.0, 1.0, 0.7)
        }
    }

    Item {
        id: menuRectangle
        enabled: mainMenuRoot.revealed
        anchors.fill: parent

        StackView {
            id: stackView
            anchors {
                top: parent.top
                left: parent.left
                right: parent.right
                bottom: parent.bottom
            }
            initialItem: mainMenuView
            delegate: StackViewDelegate {
                pushTransition: StackViewTransition {
                    PropertyAnimation {
                        target: enterItem
                        property: "y"
                        from: enterItem.height
                        to: 0
                        duration: 600
                        easing.type: Easing.InOutQuad
                    }
                    PropertyAnimation {
                        target: exitItem
                        property: "y"
                        from: 0
                        to: -exitItem.height
                        duration: 600
                        easing.type: Easing.InOutQuad
                    }
                }
                popTransition: StackViewTransition {
                    PropertyAnimation {
                        target: enterItem
                        property: "y"
                        from: -enterItem.height
                        to: 0
                        duration: 600
                        easing.type: Easing.InOutQuad
                    }
                    PropertyAnimation {
                        target: exitItem
                        property: "y"
                        from: 0
                        to: exitItem.height
                        duration: 600
                        easing.type: Easing.InOutQuad
                    }
                }
            }
        }

        Image {
            id: backButton
            anchors {
                top: parent.top
                left: parent.left
                margins: Style.baseMargin
            }

            width: Style.touchableSize
            height: width
            source: "../../images/back.png"
            enabled: stackView.depth > 1
            opacity: stackView.depth > 1

            rotation: 90

            Behavior on opacity {
                NumberAnimation {
                    duration: 1000
                    easing.type: Easing.InOutQuad
                }
            }

            MouseArea {
                anchors.fill: parent
                onPressed: {
                    if(stackView.depth > 1) {
                        stackView.pop()
                    }
                }
            }
        }
    }

    MainMenuView {
        id: mainMenuView
        visible: false
        width: parent.width
        height: parent.height

        onContinueClicked: {
            hide()
        }

        onSimulationsClicked: {
            stackView.push(simulationsView)
        }

        onAboutClicked: {
            stackView.push(aboutView)
        }

        onAdvancedClicked: {
            stackView.push(advancedView)
        }
    }

    SimulationsView {
        id: simulationsView
        visible: false
        width: parent.width
        height: parent.height

        onLoadSimulation: {
            mainMenuRoot.loadSimulation(simulation)
            stackView.pop(0)
        }
    }

    AdvancedView {
        id: advancedView
        visible: false
        width: parent.width
        height: parent.height

        onSaveSimulationClicked: {
            saveSimulationRequested()
        }

        onLoadSimulationClicked: {
            loadSimulationRequested()
        }
    }

    AboutView {
        id: aboutView
        visible: false
        width: parent.width
        height: parent.height
    }

    states: [
        State {
            when: revealed
            name: "revealed"
            PropertyChanges {
                target: menuRectangle
                opacity: 1.0
                scale: 1.0
            }
            PropertyChanges {
                target: background
                opacity: 1.0
            }
        },
        State {
            when: !revealed
            name: "hidden"
            PropertyChanges {
                target: menuRectangle
                opacity: 0.0
                scale: 0.85
            }
            PropertyChanges {
                target: background
                opacity: 0.0
            }
        }
    ]

    transitions: [
        Transition {
            from: "revealed"
            to: "hidden"
            ParallelAnimation {
                NumberAnimation {
                    properties: "opacity"
                    duration: 350
                    easing.type: Easing.InOutQuad
                }
                NumberAnimation {
                    target: menuRectangle
                    properties: "scale"
                    duration: 350
                    easing.type: Easing.InOutQuad
                }
            }
        },
        Transition {
            from: "hidden"
            to: "revealed"
            ParallelAnimation {
                NumberAnimation {
                    properties: "opacity"
                    duration: 350
                    easing.type: Easing.InOutQuad
                }
                NumberAnimation {
                    target: menuRectangle
                    properties: "scale"
                    duration: 200
                    easing.type: Easing.InOutQuad
                }
            }
        }
    ]
}

