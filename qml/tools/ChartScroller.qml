import QtQuick 2.5
import QtCharts 2.0

QtObject {
    id: chartScroller

    property real timeRange: 3

    property real value
    property real time

    property real lowPassValue: 0.0

    property real yMin: 0
    property real yMax: 0

    property real lowPassFactor: 0.0

    property LineSeries lineSeries

    onValueChanged: {
        if(isNaN(value)) {
            return;
        }
        var lowPass = 0.0
        // lowPassFilter smooths over a few tenth's of the time range
        if(lineSeries.count > 1) {
            lowPass = 1.0 - lowPassFactor * (lineSeries.at(1).x - lineSeries.at(0).x) / timeRange
        }
        lowPassValue = (1.0 - lowPass) * lowPassValue + lowPass * value
        lineSeries.append(time, lowPassValue)

        if(lineSeries.count > 0) {
            var firstPoint = lineSeries.at(0)
            var lastPoint = lineSeries.at(lineSeries.count - 1)
            if(lastPoint.x - firstPoint.x > timeRange) {
                lineSeries.remove(firstPoint.x, firstPoint.y)
            }
        }
    }
}
