import QtQuick 2.0
import QtCharts 2.1
import QtQuick.Controls 2.1
import QtQuick.Controls.Material 2.1
import QtQuick.Layouts 1.1

import Neuronify 1.0

Slider {
    id: sliderRoot
    
    property real cutValue: 0.0
    property var chart
    
    orientation: Qt.Vertical
    onValueChanged: {
        rect.update()
        var chartY = handle.mapToItem(chart_, 0, handle.height / 2).y
        cutValue = chart.mapToValue(Qt.point(0, chartY), series).y
    }
    value: 0.5
    
    Rectangle {
        id: rect
        
        x: -1000
        y: sliderRoot.handle.y + sliderRoot.handle.height / 2 - height / 2
        width: 2000
        height: 3
        color: Qt.rgba(1.0, 1.0, 1.0, 0.5)
        opacity: 1.0
        z: -9
    }
}
