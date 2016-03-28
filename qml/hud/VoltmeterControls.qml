import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1

import ".."
import "../controls"
import "qrc:/qml/style"

/*!
    \qmltype VoltMeterControls
    \inqmlmodule Neuronify
    \ingroup neuronify-hud
    \brief Contains the user controls for the voltmeter.

    The voltmeter control panel contains:
\list
 \li  A mode selection (Voltage, conductance, etc)
 \li  A "connect to all"-button which connects the voltmeter to all existing neurons.
 \li  A "disconnect from all"-button which disconnects the voltmeter from all neurons it currently is connected to.
 \li  Sliders for setting the minimum and maximum voltage.
\endlist
*/

Column {
    id: voltmeterControlsRoot
    property Item voltmeter: null

    spacing: 10

    BoundSlider {
        target: voltmeter
        property: "minimumValue"
        text: "Minimum voltage"
        unit: "mV"
        minimumValue: -250
        maximumValue: 250
    }

    BoundSlider {
        target: voltmeter
        property: "maximumValue"
        text: "Maximum voltage"
        unit: "mV"
        minimumValue: -250
        maximumValue: 250
    }
}

