import QtQuick 2.5
import QtQuick.Controls 2.0

ItemDelegate {
    id: root

    property string name
    property int index
    property Component component

    anchors {
        left: parent.left
        right: parent.right
    }

    //    height: 56
    highlighted: parent.currentIndex === index

    onClicked: {
        if(parent.currentIndex !== undefined) {
            parent.currentIndex = index
        }
    }

    font.pixelSize: 18
    font.weight: Font.Normal
    text: name

    contentItem: Text {
        text: root.text
        font: root.font
        color: "white"
        elide: Text.ElideRight
        visible: root.text
        horizontalAlignment: Text.AlignLeft
        verticalAlignment: Text.AlignVCenter
    }
}
