import QtQuick 2.5

import "qrc:/qml/hud"

import "qrc:/qml/style"

Rectangle {
    default property alias columnChildren: column.children
    anchors {
        right: parent.right
        top: parent.top
        bottom: parent.bottom
    }
    
    width: Style.touchableSize * 1.5
    color: Qt.rgba(1.0, 1.0, 1.0, 0.8);
    
    MouseArea {
        anchors.fill: parent
    }
    
    Column {
        id: column

        anchors.fill: parent        
        spacing: 0
    }
}
