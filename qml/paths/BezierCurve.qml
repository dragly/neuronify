import QtQuick 2.0

Canvas {
    id:canvas
    property color color: "cyan"
    property real lineWidth: 2.5
    property point startPoint: Qt.point(0,0)
    property point controlPoint1: Qt.point(0,40)
    property point controlPoint2: Qt.point(200,100)
    property point endPoint: Qt.point(100,100)
    antialiasing: true

    x: Math.min(startPoint.x - lineWidth / 2, endPoint.x - lineWidth / 2, controlPoint1.x - lineWidth / 2, controlPoint2.x - lineWidth / 2)
    y: Math.min(startPoint.y - lineWidth / 2, endPoint.y - lineWidth / 2, controlPoint1.y - lineWidth / 2, controlPoint2.y - lineWidth / 2)
    width: Math.max(startPoint.x, endPoint.x, controlPoint1.x, controlPoint2.x) - x + 1
    height: Math.max(startPoint.y, endPoint.y, controlPoint1.y, controlPoint2.y) - y + 1

    onLineWidthChanged: requestPaint()
    onStartPointChanged: requestPaint()
    onEndPointChanged: requestPaint()
    onControlPoint1Changed: requestPaint()
    onControlPoint2Changed: requestPaint()
    onColorChanged: requestPaint()
    onVisibleChanged: {
        if(visible) {
            requestPaint()
        }
    }

    function relativeX(x) {
        return x - canvas.x;
    }
    function relativeY(y) {
        return y - canvas.y;
    }
    function curve(ctx) {
        if (0) {
        ctx.bezierCurveTo(relativeX(canvas.controlPoint1.x), relativeY(canvas.controlPoint1.y),
                          relativeX(canvas.controlPoint2.x), relativeY(canvas.controlPoint2.y),
                          relativeX(canvas.endPoint.x), relativeY(canvas.endPoint.y));
        } else {
            ctx.bezierCurveTo(relativeX(canvas.controlPoint1.x), relativeY(canvas.controlPoint1.y),
                              relativeX(canvas.controlPoint2.x), relativeY(canvas.controlPoint2.y),
                              relativeX(canvas.endPoint.x), relativeY(canvas.endPoint.y));

        }
    }


    onPaint: {
        var ctx = canvas.getContext('2d');
        ctx.save();
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        ctx.strokeStyle = canvas.color;
        ctx.lineWidth = canvas.lineWidth;

        ctx.beginPath();
        ctx.moveTo(relativeX(startPoint.x), relativeY(startPoint.y));
        curve(ctx);
        ctx.stroke();
        ctx.restore();
    }
}
