import QtQuick 2.0
import Neuronify 1.0

import "qrc:/qml/"
import "qrc:/qml/controls"

/*!
\qmltype ResistanceControl
\inqmlmodule Neuronify
\ingroup neuronify-controls
\brief Control for membrane resistance.
*/



BoundSlider {
    property PassiveCurrent current
    target: current
    property: "resistance"
    minimumValue: 1e6
    maximumValue: 1000e6
    unitScale: 1e6
    stepSize: 10e6
    precision: 1
    text: "Membrane resistance"
    unit: "MΩ"
}
