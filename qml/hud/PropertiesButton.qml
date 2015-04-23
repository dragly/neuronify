import QtQuick 2.0

import ".."
import "../style"

Image {
    id: root

    signal clicked

    property bool revealed: false

    anchors {
        right: parent.right
        bottom: parent.bottom
        margins: Style.margin
        rightMargin: -width
    }

    width: Style.touchableSize
    height: width

    fillMode: Image.PreserveAspectFit

    source: "qrc:/images/menus/properties.png"

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
            anchors.rightMargin: Style.margin
        }
    }

    transitions: Transition {
        NumberAnimation {
            property: "anchors.rightMargin"
            duration: 400
            easing.type: Easing.InOutQuad
        }
    }
}

