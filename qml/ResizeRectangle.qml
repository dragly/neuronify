import QtQuick 2.0
import "style"

Item {
    id: resizeRectangle

    property var target: parent

    property real targetWidth: target.width
    property real targetHeight: target.width

    anchors.fill: target

    function resetAllPositions() {
        topLeft.resetPosition()
        topRight.resetPosition()
        bottomLeft.resetPosition()
        bottomRight.resetPosition()
    }

    onTargetChanged: {
        resetAllPositions()
    }

    onTargetWidthChanged: {
        resetAllPositions()
    }

    onTargetHeightChanged: {
        resetAllPositions()
    }
    
    Item {
        id: topLeft

        width: Style.touchableSize
        height: Style.touchableSize

        function resetPosition() {
            x = 0 - width / 2
            y = 0 - height / 2
        }

        Component.onCompleted: resetPosition()
        onWidthChanged: resetPosition()
        onHeightChanged: resetPosition()

        MouseArea {
            anchors.fill: parent
            drag.target: parent
            propagateComposedEvents: true
            onPositionChanged: {
                if(drag.active) {
                    var xDiff = topLeft.x + topLeft.width / 2
                    var yDiff = topLeft.y + topLeft.height / 2
                    if(target.width - xDiff < 20 || target.width - yDiff < 20) {
                        topLeft.resetPosition()
                        return
                    }
                    target.x += xDiff
                    target.y += yDiff
                    target.width -= xDiff
                    target.height -= yDiff
                    topLeft.resetPosition()
                }
            }
            onClicked: {
                if(!drag.active) {
                    mouse.accepted = false
                }
            }
        }
    }


    Item {
        id: bottomLeft

        width: Style.touchableSize
        height: Style.touchableSize

        function resetPosition() {
            x = 0 - width / 2
            y = target.height - height / 2
        }

        Component.onCompleted: resetPosition()
        onWidthChanged: resetPosition()
        onHeightChanged: resetPosition()

        MouseArea {
            anchors.fill: parent
            propagateComposedEvents: true
            drag.target: parent
            onPositionChanged: {
                if(drag.active) {
                    var xDiff = bottomLeft.x + bottomLeft.width / 2
                    var yDiff = bottomLeft.y + bottomLeft.height / 2 - target.height
                    if(target.width - xDiff < 20 || target.width + yDiff < 20) {
                        bottomLeft.resetPosition()
                        return
                    }
                    target.x += xDiff
                    target.width -= xDiff
                    target.height += yDiff
                    bottomLeft.resetPosition()
                }
            }
            onClicked: {
                if(!drag.active) {
                    mouse.accepted = false
                }
            }
        }
    }


    Item {
        id: topRight

        width: Style.touchableSize
        height: Style.touchableSize

        function resetPosition() {
            x = target.width - width / 2
            y = 0 - height / 2
        }

        Component.onCompleted: resetPosition()
        onWidthChanged: resetPosition()
        onHeightChanged: resetPosition()

        MouseArea {
            anchors.fill: parent
            drag.target: parent
            propagateComposedEvents: true
            onPositionChanged: {
                if(drag.active) {
                    var xDiff = topRight.x + topRight.width / 2 - target.width
                    var yDiff = topRight.y + topRight.height / 2
                    if(target.width + xDiff < 20 || target.width - yDiff < 20) {
                        topRight.resetPosition()
                        return
                    }
                    target.y += yDiff
                    target.width += xDiff
                    target.height -= yDiff
                    topRight.resetPosition()
                }
            }
            onClicked: {
                if(!drag.active) {
                    mouse.accepted = false
                }
            }
        }
    }


    Item {
        id: bottomRight

        width: Style.touchableSize
        height: Style.touchableSize

        function resetPosition() {
            x = target.width - width / 2
            y = target.height - height / 2
        }

        Component.onCompleted: resetPosition()
        onWidthChanged: resetPosition()
        onHeightChanged: resetPosition()

        MouseArea {
            anchors.fill: parent
            drag.target: parent
            propagateComposedEvents: true
            onPositionChanged: {
                if(drag.active) {
                    var xDiff = bottomRight.x + bottomRight.width / 2 - target.width
                    var yDiff = bottomRight.y + bottomRight.height / 2 - target.height
                    if(target.width + xDiff < 20 || target.width + yDiff < 20) {
                        bottomRight.resetPosition()
                        return
                    }
                    target.width += xDiff
                    target.height += yDiff
                    bottomRight.resetPosition()
                }
            }
            onClicked: {
                if(!drag.active) {
                    mouse.accepted = false
                }
            }
        }
    }
}
