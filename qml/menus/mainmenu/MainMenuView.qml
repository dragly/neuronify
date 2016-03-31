import QtQuick 2.0
import "../../style"

Item {
    id: mainMenuView
    property string title: "Neuronify"

    signal continueClicked
    signal newSimulationClicked
    signal simulationsClicked
    signal aboutClicked
    signal advancedClicked

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
            text: "Continue simulation"
            onClicked: {
                continueClicked()
            }
        }
        MenuButton {
            width: parent.width
            text: "New simulation"
            onClicked: {
                newSimulationClicked()
            }
        }
        MenuButton {
            width: parent.width
            text: "Select simulation"
            onClicked: {
                simulationsClicked()
            }
        }
        MenuButton {
            width: parent.width
            text: "About"
            onClicked: {
                aboutClicked()
            }
        }
        MenuButton {
            width: parent.width
            text: "Advanced options"
            onClicked: {
                advancedClicked()
            }
        }
    }
}

