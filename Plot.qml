import QtQuick 2.0

Canvas {
    id: canvasRoot
    property color strokeStyle:  Qt.darker(fillStyle, 1.4)
    property color fillStyle: "#b40000" // red
    property real alpha: 1.0
    property var data: new Array(100)
    property real minimumValue: -100.0
    property real maximumValue: 100.0
    property int paintsSinceLastReset: 100

    antialiasing: true

    onScaleChanged: requestPaint()
    onDataChanged: requestPaint()

    anchors.fill: parent

    Component.onCompleted: {
        clearData()
    }

    function clearData() {
        for(var i = 0; i < data.length; i++) {
            data[i] = -Infinity
        }
    }

    function addPoint(point) {
        var newData = data
        newData.shift()
        newData.push(point)
        data = newData
    }

    onPaint: {
        var ctx = getContext("2d")
        ctx.save()
        ctx.clearRect(0, 0, canvasRoot.width, canvasRoot.height)
        ctx.globalAlpha = canvasRoot.alpha
        ctx.strokeStyle = canvasRoot.strokeStyle
        ctx.fillStyle = canvasRoot.fillStyle
        ctx.lineWidth = canvasRoot.lineWidth
        ctx.beginPath()
        var normalizedValue = (data[0] - minimumValue) / (maximumValue - minimumValue)
        var y = canvasRoot.height - normalizedValue * canvasRoot.height
        ctx.moveTo(0, y);
        var x = 0
        var started = false
        for(var i in data) {
            normalizedValue = (data[i] - minimumValue) / (maximumValue - minimumValue)
            y = canvasRoot.height - normalizedValue * canvasRoot.height

            if(!started) {
                ctx.moveTo(x,y);
            } else {
                ctx.lineTo(x,y);
            }

            if(data[i] > -9999) {
                started = true
            }

            x += 1 * canvasRoot.width / data.length;
        }
        ctx.stroke();
        ctx.restore();
        paintsSinceLastReset += 1
    }
}
