import QtQuick 2.5
import QtQuick.Controls 2.0

FileMenuDelegate {
    id: root

    property string name
    property string identifier
    property Component component

    anchors {
        left: parent.left
        right: parent.right
    }

    text: name
}
