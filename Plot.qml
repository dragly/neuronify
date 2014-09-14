import QtQuick 2.0

Canvas {
    id: canvasRoot
    property color strokeStyle:  Qt.darker(fillStyle, 1.4)
    property color fillStyle: "#b40000" // red
    property real alpha: 1.0
    property var data: new Array(100)

    Component.onCompleted: {
        for(var i = 0; i < data.length; i++) {
            data[i] = -9999.0
        }
    }

    antialiasing: true

    onScaleChanged: requestPaint()
    onDataChanged: requestPaint()
    anchors {
        bottom: parent.bottom
        left: parent.left
        right: parent.right
    }
    height: parent.height / 2.0

    function addPoint(point) {
        var newData = data
        newData.shift()
        newData.push(point)
        data = newData
    }

    onPaint: {
        var ctx = getContext("2d")
        ctx.save();
        ctx.clearRect(0, 0, canvasRoot.width, canvasRoot.height);
        ctx.globalAlpha = canvasRoot.alpha;
        ctx.strokeStyle = canvasRoot.strokeStyle;
        ctx.fillStyle = canvasRoot.fillStyle;
        ctx.lineWidth = canvasRoot.lineWidth;
        ctx.beginPath();
        var minValue = -100
        var maxValue = 100
        var normalizedValue = (data[0] - minValue) / (maxValue - minValue)
        var y = canvasRoot.height - normalizedValue * canvasRoot.height
        ctx.moveTo(0, y);
        var x = 0
        var started = false
        for(var i in data) {
            normalizedValue = (data[i] - minValue) / (maxValue - minValue)
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
    }
}
