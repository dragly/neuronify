import QtQuick 2.2
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.0
import Qt.labs.settings 1.0

ApplicationWindow {
    id: applicationWindow1
    visible: true
    width: 1136
    height: 640
    title: qsTr("Neuronify")

    Settings {
        id: settings
        property alias width: applicationWindow1.width
        property alias height: applicationWindow1.height
        property alias x: applicationWindow1.x
        property alias y: applicationWindow1.y

    }

    Neuronify {
        anchors.fill: parent
    }


}
