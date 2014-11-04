import QtQuick 2.2
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.0

/*
  TODO:
    * Create synapses
    * Remove connection parameters and add area and length parameters to compartments
*/

ApplicationWindow {
    id: applicationWindow1
    visible: true
    width: 1280
    height: 800
    title: qsTr("Nestify")

    Nestify {
        anchors.fill: parent
    }
}
