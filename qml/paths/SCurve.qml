import QtQuick 2.0

BezierCurve {
    controlPoint1: Qt.point((startPoint.x + endPoint.x) / 2, startPoint.y)
    controlPoint2: Qt.point((startPoint.x + endPoint.x) / 2, endPoint.y)
}
