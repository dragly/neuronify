import QtCharts 2.0
import "tools"

LineSeries {
    id: seriesRoot
    property alias timeRange: scroller.timeRange

    function addPoint(x, y) {
        scroller.append(x, y)
    }

    ChartScroller {
        id: scroller
        series: seriesRoot
    }

    pointsVisible: false
    visible: false
}
