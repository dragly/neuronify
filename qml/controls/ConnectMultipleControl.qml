import QtQuick 2.6
import QtQuick.Controls 2.1

import Neuronify 1.0

import ".."
import "../controls"
import "../style"

PropertiesItem {
    property Node node
    property bool fromEnabled: true
    property bool toEnabled: true

    text: "Connect multiple"
    Button {
        visible: toEnabled
        text: "Connect to this"
        onClicked: {
            node.startConnectMultipleToThis();
        }
    }
    Button {
        visible: fromEnabled
        text: "Connect from this"
        onClicked: {
            node.startConnectMultipleFromThis();
        }
    }
}
