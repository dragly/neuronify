import QtQuick 2.7
import QtCharts 2.1
import QtQuick.Controls 2.1
import QtQuick.Controls.Material 2.1
import QtQuick.Layouts 1.1

import Neuronify 1.0

Item {
    id: sliderRoot

    property real cutValue: 0.0
    property var chart
    property var series
    property alias text: label.text
    property alias value: slider.value

    property real _sliderMinimum: 0.0
    property real _sliderMaximum: 0.0

    property var next: null
    property alias field: thresholdField

    function refresh() {
        dummySlider.value = 1
        var maxPoint = dummySlider.handle.mapToItem(chart, 0, dummySlider.handle.height / 2)
        _sliderMaximum = chart.mapToValue(maxPoint, series).y
        dummySlider.value = 0
        var minPoint = dummySlider.handle.mapToItem(chart, 0, dummySlider.handle.height / 2)
        _sliderMinimum = chart.mapToValue(minPoint, series).y
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
            var chartY = handle.mapToItem(chart, 0, handle.height / 2).y
            cutValue = chart.mapToValue(Qt.point(0, chartY), series).y
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
            var min = _sliderMinimum
            var max = _sliderMaximum
            var value = (Number(text) - min) / (max - min)
            console.log(min, max, value)
            slider.value = value
        }
        Binding {
            target: thresholdField
            property: "text"
            value: cutValue.toFixed(1)
        }
    }
}
