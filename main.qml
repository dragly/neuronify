import QtQuick 2.2
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.0

/*
  TODO:
  * Select multiple items
  * Copy+paste
  * Save state
  * Motifs
  * Change background color
*/

ApplicationWindow {
    id: applicationWindow1
    visible: true
    width: 1136
    height: 640
    title: qsTr("Neuronify")

    Nestify {
        anchors.fill: parent
    }
}
