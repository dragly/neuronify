import QtQuick 2.0
import "../../style"


Item {
    id: root

//    anchors {
//        top: parent.top
//        left: parent.left
//    }
    width: Style.touchableSize * 1.5
    height: width

    enabled: revealed
    state: revealed ? "revealed" : "hidden"
    property bool revealed: true

    signal clicked

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
    Image {

        anchors.centerIn: parent
        width: Style.touchableSize
        height: width

        source: "qrc:/images/tools/mainmenu.png"
    }


    MouseArea {
        anchors.fill: parent
        onClicked: {
            root.clicked()
        }
    }
}

