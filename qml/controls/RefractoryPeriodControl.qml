import QtQuick 2.0
import Neuronify 1.0

import "qrc:/"
import "qrc:/qml/"
import "qrc:/qml/controls"
import "qrc:/qml/neurons"

/*!
\qmltype RefractoryPeriodControl
\inqmlmodule Neuronify
\ingroup neuronify-controls
\brief Control for refractory period of a neuron.
*/



BoundSlider {
    property NeuronEngine engine
    target: engine
    property: "refractoryPeriod"
    text: "Refractory period"
    unit: "ms"
    minimumValue: 0.0e-3
    maximumValue: 50e-3
    unitScale: 1e-3
    stepSize: 1e-3
    precision: 1
}
