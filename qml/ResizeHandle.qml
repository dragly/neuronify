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
    
    width: 64
    height: 64
    
    MouseArea {
        id: mouseArea
        property point previousPosition
        anchors.fill: parent
        propagateComposedEvents: true
        onPositionChanged: {
            var deltaX = mouse.x - previousPosition.x;
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
            }
            var deltaY = mouse.y - previousPosition.y;
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
            }
        }
        onPressed: {
            previousPosition.x = mouse.x
            previousPosition.y = mouse.y
        }
    }
}
