import QtQuick 2.0

Rectangle {
    property bool revealed: false
    anchors {
        right: parent.right
        top: parent.top
        rightMargin: revealed ? 0.0 : -width
        bottom: parent.bottom
    }

    color: "#f7fbff"
    width: parent.width * 0.25

    border.color: "#9ecae1"
    border.width: 1.0

    Behavior on anchors.rightMargin {
        NumberAnimation {
            duration: 350
            easing.type: Easing.InOutCubic
        }
    }

    MouseArea {
        anchors.fill: parent
    }
}
