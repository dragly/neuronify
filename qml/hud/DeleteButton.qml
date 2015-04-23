import QtQuick 2.0

import ".."
import "../style"

Image {
    id: root

    signal clicked

    property bool revealed: false

    anchors {
        left: parent.left
        bottom: parent.bottom
        margins: Style.margin
        leftMargin: -width
    }

    width: Style.touchableSize
    height: width

    fillMode: Image.PreserveAspectFit

    source: "../../images/delete.png"

    MouseArea {
        anchors.fill: parent
        onClicked: {
            root.clicked()
        }
    }

    states: State {
        when: revealed
        PropertyChanges {
            target: root
            anchors.leftMargin: Style.margin
        }
    }

    transitions: Transition {
        NumberAnimation {
            property: "anchors.leftMargin"
            duration: 400
            easing.type: Easing.InOutQuad
        }
    }
}

