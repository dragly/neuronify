import QtQuick 2.0
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

    Image {
        id: logo
        anchors {
            right: parent.right
            verticalCenter: parent.verticalCenter
            rightMargin: Style.touchableSize
        }
        width: mainMenuView.width * 0.4
        height: mainMenuView.height * 0.5
        fillMode: Image.PreserveAspectFit
        source: "qrc:/images/logo/mainMenuLogo.png"
    }

    Column {
        anchors {
            left: parent.left
            leftMargin: Style.touchableSize
            verticalCenter: parent.verticalCenter
        }
        width: mainMenuView.width * 0.4
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

