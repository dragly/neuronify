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
    minimumValue: 0.0
    maximumValue: 100e-6
    unitScale: 1e-6
    stepSize: 1e-7
    precision: 1
    text: "Adaptation"
    unit: "uS"
}
