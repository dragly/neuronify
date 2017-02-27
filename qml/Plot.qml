import QtCharts 2.0
import "tools"

LineSeries {
    id: seriesRoot
    property alias timeRange: scroller.timeRange

    function addPoint(x, y) {
        scroller.append(x, y)
    }

    pointsVisible: false
    useOpenGL: false

    ChartScroller {
        id: scroller
        series: seriesRoot
    }
}
