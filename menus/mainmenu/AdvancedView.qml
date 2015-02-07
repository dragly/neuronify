import QtQuick 2.0
import "../../style"

Item {
    id: mainMenuView

    signal saveSimulationClicked
    signal loadSimulationClicked

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

        MenuButton {
            height: mainMenuView.width * 0.07
            width: parent.width
            text: "Save simulation"
            onClicked: {
                saveSimulationClicked()
            }
        }
        MenuButton {
            height: mainMenuView.width * 0.07
            width: parent.width
            text: "Load simulation"
            onClicked: {
                loadSimulationClicked()
            }
        }
    }
}
