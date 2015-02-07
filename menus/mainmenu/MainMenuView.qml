import QtQuick 2.0
import "../../style"

Item {
    id: mainMenuView
    property string title: "Atomify"

    signal continueClicked
    signal simulationsClicked
    signal aboutClicked
    signal advancedClicked

    width: 200
    height: 100

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
            height: mainMenuView.width * 0.07
            width: parent.width
            text: "Continue simulation"
            onClicked: {
                continueClicked()
            }
        }
        MenuButton {
            height: mainMenuView.width * 0.07
            width: parent.width
            text: "Select simulation"
            onClicked: {
                simulationsClicked()
            }
        }
        MenuButton {
            height: mainMenuView.width * 0.07
            width: parent.width
            text: "About"
            onClicked: {
                aboutClicked()
            }
        }
        MenuButton {
            height: mainMenuView.width * 0.07
            width: parent.width
            text: "Advanced options"
            onClicked: {
                advancedClicked()
            }
        }
    }
}
