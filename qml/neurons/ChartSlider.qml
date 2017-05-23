import QtQuick 2.7
import QtCharts 2.1
import QtQuick.Controls 2.1
import QtQuick.Controls.Material 2.1
import QtQuick.Layouts 1.1

import Neuronify 1.0

Item {
    id: root

    property var chart
    property var series
    property alias text: label.text
    property alias value: slider.value

    property var next: null
    property alias field: thresholdField
    property bool ready: false

    function refresh() {
        dummySlider.value = 1
        var maxPoint = dummySlider.handle.mapToItem(chart, 0, dummySlider.handle.height / 2)
        var max = chart.mapToValue(maxPoint, series).y
        dummySlider.value = 0
        var minPoint = dummySlider.handle.mapToItem(chart, 0, dummySlider.handle.height / 2)
        var min = chart.mapToValue(minPoint, series).y
        if(max > min) {
            slider.to = max
            slider.from = min
            ready = true
        }
    }

    implicitWidth: Math.max(label.implicitWidth, fieldMetrics.width)

    onWidthChanged: refresh()
    onHeightChanged: refresh()
    Component.onCompleted: refresh()

    Connections {
        target: chart
        onWidthChanged: refresh()
        onHeightChanged: refresh()
        onXChanged: refresh()
        onYChanged: refresh()
    }

    Label {
        id: label
        anchors {
            top: parent.top
            horizontalCenter: parent.horizontalCenter
        }
    }

    Slider {
        id: dummySlider
        visible: false
        anchors.fill: slider
        orientation: Qt.Vertical
    }

    Slider {
        id: slider
        anchors {
            top: label.bottom
            bottom: thresholdField.top
            horizontalCenter: parent.horizontalCenter
        }

        orientation: Qt.Vertical
        onValueChanged: {
            rect.update()
        }
        value: 0.5
        hoverEnabled: true

        Rectangle {
            id: rect

            x: -1000
            y: slider.handle.y + slider.handle.height / 2 - height / 2
            width: 2000
            height: 3
            color: {
                var col = Material.accent
                return Qt.rgba(col.r, col.g, col.b, 0.2)
            }

            opacity: slider.hovered || thresholdField.activeFocus ? 1.0 : 0.0
            z: -1

            Behavior on opacity {
                NumberAnimation {
                    duration: 200
                    easing.type: Easing.InOutQuad
                }
            }
        }
    }

    TextMetrics {
        id: fieldMetrics
        font: thresholdField.font
        text: "-00.00"
    }

    TextField {
        id: thresholdField
        anchors {
            bottom: parent.bottom
            left: parent.left
            right: parent.right
        }
        horizontalAlignment: Text.AlignHCenter
//        validator: DoubleValidator {
//            decimals: 1
//        }asfasgasd
        KeyNavigation.tab: next

        onEditingFinished: {
            slider.value = Number(text)
        }
        Binding {
            target: thresholdField
            property: "text"
            value: root.value.toFixed(1)
        }
    }
}
