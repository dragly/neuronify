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
                //                height: mainMenuRoot.width * 0.4
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
                //        MenuButton {
                //            width: parent.width
                //            text: "Select"
                //            onClicked: {
                //                simulationsClicked()
                //            }
                //        }
                MenuButton {
                    width: parent.width
                    text: "Save"
                    onClicked: {
                        saveView.isSave = true
                        saveClicked()
                    }
                }
                MenuButton {
                    width: parent.width
                    text: "Load"
                    onClicked: {
                        saveView.isSave = false
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
                //        MenuButton {
                //            width: parent.width
                //            text: "Advanced options"
                //            onClicked: {
                //                advancedClicked()
                //            }
                //        }
            }
        }

        Item {
            width: 1
            height: 1
            Layout.fillHeight: true
            Layout.fillWidth: true
            Image {
                id: logo
                anchors {
                    fill: parent
                    margins: 0.2 * parent.width
                }
                fillMode: Image.PreserveAspectFit
                source: "qrc:/images/logo/mainMenuLogo.png"
                smooth: true
                antialiasing: true
            }
        }
    }
}

