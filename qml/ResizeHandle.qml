import QtQuick 2.0
import "style"

Item {
    id: topLeft
    
    property var target
    property bool ignoreChange: false

    property real xFactor: 0.0
    property real yFactor: 0.0
    property real widthFactor: 0.0
    property real heightFactor: 0.0

    property bool forceSquare: false
    
    width: 96
    height: 96

    Rectangle {
        anchors.centerIn: parent
        width: parent.width * 0.5
        height: width
        color: Style.border.color
        opacity: 0.4
        radius: width * 0.5
    }
    
    MouseArea {
        id: mouseArea
        property point previousPosition
        anchors.fill: parent
        onPositionChanged: {
            var deltaX = mouse.x - previousPosition.x;
            var deltaY = mouse.y - previousPosition.y;

            var biggestIsX = Math.abs(deltaX) > Math.abs(deltaY);
            if(forceSquare) {
                if(biggestIsX) {
                    deltaX = widthFactor * heightFactor * deltaY;
                } else {
                    deltaY = widthFactor * heightFactor * deltaX;
                }
            }

            if(Math.abs(deltaX) > target.snapGridSize) {
                var newX = target.x + xFactor * deltaX;
                newX = Math.round(newX / target.snapGridSize) * target.snapGridSize;
                var xDiff = target.x - newX;
                target.x = newX;

                var newWidth = target.width + widthFactor * deltaX;
                newWidth = Math.round(newWidth / target.snapGridSize) * target.snapGridSize;
                if(newWidth > Math.max(target.snapGridSize, 32)) {
                    target.width = newWidth;
                }
                if(forceSquare && !biggestIsX) {
                    previousPosition.y -= deltaY;
                }
            }
            if(Math.abs(deltaY) > target.snapGridSize) {
                var newY = target.y + yFactor * deltaY;
                newY = Math.round(newY / target.snapGridSize) * target.snapGridSize;
                var yDiff = target.y - newY;
                target.y = newY;

                var newHeight = target.height + heightFactor * deltaY;
                newHeight = Math.round(newHeight / target.snapGridSize) * target.snapGridSize;
                if(newHeight > Math.max(target.snapGridSize, 32)) {
                    target.height = newHeight;
                }
                if(forceSquare && biggestIsX) {
                    previousPosition.x -= deltaX;
                }
            }
        }
        onPressed: {
            previousPosition.x = mouse.x
            previousPosition.y = mouse.y
        }
    }
}
