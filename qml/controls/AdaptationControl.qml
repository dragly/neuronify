import QtQuick 2.0
import Neuronify 1.0

import "qrc:/qml/"
import "qrc:/qml/controls"
/*!
\qmltype AdaptationControl
\inqmlmodule Neuronify
\ingroup neuronify-controls
\brief Control for adaptation current.
*/


BoundSlider {
    property AdaptationCurrent current
    target: current
    property: "adaptation"
    minimumValue: 0.0e-9
    maximumValue: 100e-9
    unitScale: 1e-9
    stepSize: 1e-10
    precision: 1
    text: "Adaptation"
    unit: "nS"
}
