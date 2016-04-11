import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import QtQuick.Controls.Styles 1.1

import Neuronify 1.0

import "qrc:/qml"
import "qrc:/qml/neurons"

/*!
\qmltype RestControl
\inqmlmodule Neuronify
\ingroup neuronify-controls
\brief Control for resetting potential.
*/



Column {
    id: root
    property NeuronEngine engine: null

    width: parent.width

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
