import QtQuick 2.0

Rectangle {
    id: handle
    property alias containsMouse: mouseArea.containsMouse

    anchors {
        horizontalCenter: parent.left
        verticalCenter: parent.top
    }
    width: parent.height / 2
    height: width
    radius: width / 2
    color: "#c6dbef"
    border.width: width * 0.1
    border.color: "#f7fbff"
    
    Image {
        anchors.fill: parent
        anchors.margins: parent.width * 0.1
        source: "qrc:/images/transform-move.png"
        smooth: true
        antialiasing: true
    }
    
    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        drag.target: handle.parent.dragProxy
        onPressed: {
            handle.parent.dragging = true
            handle.parent.dragStarted(handle.parent)
        }
        onClicked: {
            handle.parent.clicked(handle.parent, mouse)
        }        
        onReleased: {
            handle.parent.dragging = false
        }
    }
}
