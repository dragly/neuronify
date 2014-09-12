import QtQuick 2.0

Canvas {
    id: canvasRoot
    property color strokeStyle:  Qt.darker(fillStyle, 1.4)
    property color fillStyle: "#b40000" // red
    property real alpha: 1.0
    property var data: new Array(1000)
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
        ctx.moveTo(0,100 - data[0]);
        var x = 0;
        for(var i in data) {
            var y = data[i];
            ctx.lineTo(x,100 - y);
            x += 1;
        }
        ctx.stroke();
        ctx.restore();
    }
}
