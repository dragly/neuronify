import QtQuick 2.0
import "../../style"

Item {
    id: mainMenuButton
    property bool revealed: true

    anchors {
        top: parent.top
        left: parent.left
    }
    width: Style.touchableSize * 2.5
    height: width

    enabled: revealed
    state: revealed ? "revealed" : "hidden"

    states: [
        State {
            name: "hidden"
            PropertyChanges {
                target: mainMenuButton
                opacity: 0.0
            }
        },
        State {
            name: "revealed"
            PropertyChanges {
                target: mainMenuButton
                opacity: 1.0
            }
        }
    ]

    transitions: [
        Transition {
            NumberAnimation {
                properties: "opacity"
                duration: 200
                easing.type: Easing.InOutQuad
            }
        }
    ]

    Image {
        anchors {
            fill: parent
            margins: parent.width * 0.2
        }
        source: "../../images/menus/mainmenu.png"
    }

    MouseArea {
        anchors.fill: parent
        onClicked: {
            mainMenu.revealed = true
        }
    }
}

