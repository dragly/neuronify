import QtQuick 2.6
import QtQuick.Controls 2.1

import Neuronify 1.0

import ".."
import "../controls"
import "../neurons"
import "../style"

PropertiesItem {
    text: "Reset"

    property NodeEngine engine: null

    Button {
        text: "Reset dynamics"
        onClicked: {
            engine.resetDynamics()
        }
    }

    Button {
        text: "Reset properties"
        onClicked: {
            engine.resetProperties()
        }
    }
}
