import QtQuick 2.0

import "../hud"
import "../style"

CreationItem {

    width: Style.touchableSize
    height: width

    source: "qrc:/qml/neurons/AdaptationNeuron.qml"

    Rectangle {
        anchors.fill: parent
        color: "green"
        border.color: "#6baed6"
        border.width: 2.0
        radius: width / 2.0
    }
}

