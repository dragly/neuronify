import QtQuick 2.0
import QtQuick.Layouts 1.1

import "../../style"

MainMenuPage {
    id: mainMenuView
    signal continueClicked
    signal newSimulationClicked
    signal simulationsClicked
    signal aboutClicked
    signal advancedClicked
    signal saveClicked
    signal loadClicked

    width: 200
    height: 100

    GridLayout {
        anchors.fill: parent
        columns: mainMenuView.width > mainMenuView.height ? 2 : 1

        Item {
            width: 1
            height: 1
            Layout.fillHeight: true
            Layout.fillWidth: true
            Column {
                anchors {
                    left: parent.left
                    right: parent.right
                    verticalCenter: parent.verticalCenter
                    leftMargin: 0.1 * parent.width
                    rightMargin: 0.1 * parent.width
                }
                spacing: mainMenuView.width * 0.02

                MenuButton {
                    width: parent.width
                    text: "Continue"
                    onClicked: {
                        continueClicked()
                    }
                }
                MenuButton {
                    width: parent.width
                    text: "New"
                    onClicked: {
                        simulationsClicked()
                    }
                }
                MenuButton {
                    width: parent.width
                    text: "Save"
                    onClicked: {
                        saveClicked()
                    }
                }
                MenuButton {
                    width: parent.width
                    text: "Load"
                    onClicked: {
                        loadClicked()
                    }
                }

                MenuButton {
                    width: parent.width
                    text: "About"
                    onClicked: {
                        aboutClicked()
                    }
                }
            }
        }

        Item {
            width: 1
            height: 1
            Layout.fillHeight: true
            Layout.fillWidth: true
            Text {
                id: landscapeText
                anchors {
                    left: parent.left
                    right: parent.right
                    top: parent.top
                    margins: 0.1 * parent.width
                }
                horizontalAlignment: Text.AlignHCenter
                visible: mainMenuView.width < mainMenuView.height ? true : false
                text: "Neuronify is best viewed in landscape orientation"
                font: Style.font
                color: "#777"
            }
            Image {
                id: logo
                anchors {
                    left: parent.left
                    right: parent.right
                    top: mainMenuView.width < mainMenuView.height ? landscapeText.top : parent.top
                    bottom: parent.bottom
                    leftMargin: 0.1 * parent.width
                    rightMargin: 0.1 * parent.width
                    topMargin: 0.1 * parent.width
                    bottomMargin: 0.1 * parent.width
                }
                fillMode: Image.PreserveAspectFit
                source: "qrc:/images/logo/mainMenuLogo.png"
                smooth: true
                antialiasing: true
            }
        }
    }
}

