import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1

import "qrc:/qml"
import "qrc:/qml/controls"
import "qrc:/qml/style"

/*!
    \qmltype MeterControls
    \inqmlmodule Neuronify
    \ingroup neuronify-hud
    \brief Contains the user controls for the meters.

    The meter control panel contains:
\list
 \li  A "connect to all"-button which connects the meter to all existing neurons.
 \li  A "disconnect from all"-button which disconnects the meter from all neurons it currently is connected to.
 \li  Sliders for setting the minimum and maximum values.
\endlist
*/

Column {
    id: meterControlsRoot
    property Item meter: null
    property double sliderMinimum: -250
    property double sliderMaximum: 250
    property string unit: ""
    property string meterType: ""

    spacing: Style.spacing

    onMeterChanged: {
        if(!meterControlsRoot.meter) {
            return
        }

    }

    BoundSlider {
        target: meter
        property: "minimumValue"
        text: "Minimum " + meterControlsRoot.meterType
        unit: meterControlsRoot.unit
        minimumValue: meterControlsRoot.sliderMinimum
        maximumValue: meterControlsRoot.sliderMaximum
    }

    BoundSlider {
        target: meter
        property: "maximumValue"
        text: "Maximum " + meterControlsRoot.meterType
        unit: meterControlsRoot.unit
        minimumValue: meterControlsRoot.sliderMinimum
        maximumValue: meterControlsRoot.sliderMaximum
    }

    Item {
        anchors {
            left: parent.left
            right: parent.right
        }
        height: Math.max(switchText.height, showLegendSwitch.height)

        Label {
            id: switchText

            anchors {
                left: parent.left
                verticalCenter: parent.verticalCenter
            }
            text: "Show legend"
        }

        Switch {
            id: showLegendSwitch
            anchors {
                right: parent.right
                verticalCenter: parent.verticalCenter
            }
            checked: meter.showLegend
        }

        Binding {
            target: showLegendSwitch
            property: "checked"
            value: meter.showLegend
        }

        Binding {
            target: meter
            property: "showLegend"
            value: showLegendSwitch.checked
        }

        MouseArea {
            anchors.fill: parent
            propagateComposedEvents: true
            onClicked: {
                showLegendSwitch.checked = !showLegendSwitch.checked
            }
        }
    }

}

