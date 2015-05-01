import QtQuick 2.0

Item {
    id: resizeRectangle

    property var target: parent
    anchors.fill: target
    
    Item {
        id: topLeft

        width: 100
        height: 100

        function resetPosition() {
            x = 0 - width / 2
            y = 0 - height / 2
        }

        Component.onCompleted: resetPosition()

        MouseArea {
            anchors.fill: parent
            drag.target: parent
            onPositionChanged: {
                if(drag.active) {
                    var xDiff = topLeft.x + topLeft.width / 2
                    var yDiff = topLeft.y + topLeft.height / 2
                    target.x += xDiff
                    target.y += yDiff
                    target.width -= xDiff
                    target.height -= yDiff
                    topLeft.resetPosition()
                }
            }
        }
    }

    Item {
        id: bottomLeft

        width: 100
        height: 100

        function resetPosition() {
            x = 0 - width / 2
            y = target.height - height / 2
        }

        Component.onCompleted: resetPosition()

        MouseArea {
            anchors.fill: parent
            drag.target: parent
            onPositionChanged: {
                if(drag.active) {
                    var xDiff = bottomLeft.x + bottomLeft.width / 2
                    var yDiff = bottomLeft.y + bottomLeft.height / 2 - target.height
                    target.x += xDiff
                    target.width -= xDiff
                    target.height += yDiff
                    bottomLeft.resetPosition()
                }
            }
        }
    }

    Item {
        id: topRight

        width: 100
        height: 100

        function resetPosition() {
            x = target.width - width / 2
            y = 0 - height / 2
        }

        Component.onCompleted: resetPosition()

        MouseArea {
            anchors.fill: parent
            drag.target: parent
            onPositionChanged: {
                if(drag.active) {
                    var xDiff = topRight.x + topRight.width / 2 - target.width
                    var yDiff = topRight.y + topRight.height / 2
                    target.y += yDiff
                    target.width += xDiff
                    target.height -= yDiff
                    topRight.resetPosition()
                }
            }
        }
    }

    Item {
        id: bottomRight

        width: 100
        height: 100

        function resetPosition() {
            x = target.width - width / 2
            y = target.height - height / 2
        }

        Component.onCompleted: resetPosition()

        MouseArea {
            anchors.fill: parent
            drag.target: parent
            onPositionChanged: {
                if(drag.active) {
                    var xDiff = bottomRight.x + bottomRight.width / 2 - target.width
                    var yDiff = bottomRight.y + bottomRight.height / 2 - target.height
                    target.width += xDiff
                    target.height += yDiff
                    bottomRight.resetPosition()
                }
            }
        }
    }
}
