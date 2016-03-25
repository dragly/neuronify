import QtCharts 2.0
import "tools"

LineSeries {
    id: seriesRoot
    property alias scroller: seriesScroller
    property alias timeRange: seriesScroller.timeRange

    ChartScroller {
        id: seriesScroller
        lineSeries: seriesRoot
    }

    pointsVisible: false
    visible: false
}
