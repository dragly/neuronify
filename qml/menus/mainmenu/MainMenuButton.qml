import QtQuick 2.0
import "../../style"

Image {
    id: root
    property bool revealed: true

    signal clicked

    anchors {
        top: parent.top
        left: parent.left
        margins: Style.margin
    }
    width: Style.touchableSize
    height: width

    enabled: revealed
    state: revealed ? "revealed" : "hidden"

    source: "qrc:/images/menus/mainmenu.png"

    states: [
        State {
            name: "hidden"
            PropertyChanges {
                target: root
                opacity: 0.0
            }
        },
        State {
            name: "revealed"
            PropertyChanges {
                target: root
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

    MouseArea {
        anchors.fill: parent
        onClicked: {
            root.clicked()
        }
    }
}

