import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import QtGraphicalEffects 1.0

import "../../style"
import "../../io"

Item {
    id: mainMenuRoot

    signal continueClicked
    signal newClicked
    signal loadSimulation(var simulation)
    signal saveSimulation(var simulation)
    signal requestScreenshot(var callback)
    signal saveSimulationRequested
    signal loadSimulationRequested

    property bool revealed: true
    property var blurSource: null

    width: 100
    height: 62

    MouseArea {
        enabled: mainMenuRoot.revealed
        anchors.fill: parent
        onWheel: {
            wheel.accepted = true
        }
    }

    Item {
        id: background
        anchors.fill: parent

        Rectangle {
            anchors.fill: parent
            color: Style.color.background
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
                margins: Style.margin
            }

            width: Style.touchableSize
            height: width
            source: "qrc:/images/back.png"
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
            mainMenuRoot.continueClicked()
        }

        onNewSimulationClicked: {
            mainMenuRoot.newClicked()
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

        onSimulationClicked: {
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
            //saveSimulationRequested()
            stackView.push(saveView)
        }

        onLoadSimulationClicked: {
            stackView.push(saveView)
        }
    }

    AboutView {
        id: aboutView
        visible: false
        width: parent.width
        height: parent.height
    }

    SaveView {
        id: saveView
        visible: false
        width: parent.width
        height: parent.height
        onLoad: loadSimulation(filename)
        onSave: saveSimulation(filename)
        onRequestScreenshot: mainMenuRoot.requestScreenshot(callback)
    }

    states: [
        State {
            when: !mainMenuRoot.revealed
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
        }
    ]
}

