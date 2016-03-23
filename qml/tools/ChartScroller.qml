import QtQuick 2.5
import QtCharts 2.0

QtObject {
    id: chartScroller

    property real timeRange: 3
    property var series

    function append(time, value) {
        if(isNaN(value)) {
            return;
        }
        series.append(time, value)

        if(series.count > 0) {
            var firstPoint = series.at(0)
            var lastPoint = series.at(series.count - 1)
            if(lastPoint.x - firstPoint.x > timeRange) {
                series.remove(firstPoint.x, firstPoint.y)
            }
        }
    }
}
