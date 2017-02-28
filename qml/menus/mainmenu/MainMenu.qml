import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import QtGraphicalEffects 1.0

import ".."
import "../../io"
import "../../style"

Item {
    id: mainMenuRoot

    signal continueClicked
    signal newClicked
    signal loadSimulation(var simulation)
    signal saveSimulation(var simulation)
    signal requestScreenshot(var callback)
    signal saveSimulationRequested
    signal loadSimulationRequested

    property bool revealed: false

    state: revealed ? "revealed" : "hidden"

    width: 100
    height: 62

    onRevealedChanged: {
        if (revealed) {
            focus = true
        }
        else {
            focus = false
        }
    }

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


        Image {
            id: backButton
            anchors {
                top: parent.top
                left: parent.left
                topMargin: Style.margin
                leftMargin: Style.margin
            }

            asynchronous: true
            width: Style.touchableSize
            height: width
            source: "qrc:/images/tools/back.png"
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

        Heading {
            anchors {
                top: parent.top
                horizontalCenter: parent.horizontalCenter
                margins: Style.margin
            }

            text: stackView.currentItem ? stackView.currentItem.title : ""
        }


        StackView {
            id: stackView
            anchors {
                top: backButton.bottom
                left: parent.left
                right: parent.right
                bottom: parent.bottom
            }



            clip: true
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
    }



    Component {
        id: mainMenuView
        MainMenuView {
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
            onSaveClicked: {
                stackView.push(saveView);
                stackView.currentItem.isSave = true;
            }
            onLoadClicked: {
                stackView.push(saveView);
                stackView.currentItem.isSave = false;
            }
        }
    }

    Component {
        id: simulationsView
        SimulationsView {
            visible: false
            width: parent.width
            height: parent.height

            onSimulationClicked: {
                mainMenuRoot.loadSimulation(simulation)
                stackView.pop(0)
            }
        }
    }

    Component {
        id: aboutView
        AboutView {
            visible: false
            width: parent.width
            height: parent.height
        }
    }

    Component {
        id: saveView
        SaveView {
            visible: false
            width: parent.width
            height: parent.height
            onLoad: {
                loadSimulation(filename)
                stackView.pop()
            }
            onSave: {
                saveSimulation(filename)
                stackView.pop()
            }
            onRequestScreenshot: mainMenuRoot.requestScreenshot(callback)
        }
    }

    states: [
        State {
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
        },
        State {
            name: "revealed"
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


    Keys.onPressed: {
        if(event.key === Qt.Key_Back || event.key === Qt.Key_Escape) {
            if(stackView.depth > 1){
                stackView.pop();
            } else {
                revealed = false
            }

        }
        event.accepted = true
    }

}

