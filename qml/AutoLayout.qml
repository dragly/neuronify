import QtQuick 2.0


Item {
    property var connections: []
    property var entities: []
    property real lastOrganizeTime: Date.now()
    property real springLength: 100

    function organize() {
        if(!enabled) {
            return
        }

        var currentOrganizeTime = Date.now()
        var dt = Math.min(0.032, (currentOrganizeTime - lastOrganizeTime) / 1000.0)
        var anyDragging = false

        for(var i in entities) {
            var item = entities[i]
            item.velocity = Qt.vector2d(0,0)
            if(item.dragging) {
                anyDragging = true
            }
        }

        for(var i in connections) {
            var connection = connections[i]
            var source = connection.itemA
            var target = connection.itemB
            var totalSpringLength = source.width / 2.0 + target.width / 2.0 + springLength
            var sourceCenter = itemCenter(source)
            var targetCenter = itemCenter(target)
            var xDiff = sourceCenter.x - targetCenter.x
            var yDiff = sourceCenter.y - targetCenter.y
            var length = Math.sqrt(xDiff*xDiff + yDiff*yDiff)
            var lengthDiff = length - totalSpringLength
            var xDelta = lengthDiff * xDiff / length
            var yDelta = lengthDiff * yDiff / length
            var kFactor = lengthDiff > 0 ? 0.015 : 0.005
            var k = kFactor * neuronifyRoot.width
            if(!source.dragging) {
                source.velocity.x -= 0.5 * k * xDelta
                source.velocity.y -= 0.5 * k * yDelta
            }
            if(!target.dragging) {
                target.velocity.x += 0.5 * k * xDelta
                target.velocity.y += 0.5 * k * yDelta
            }
        }

        for(var i = 0; i < entities.length; i++) {
            var minDistance = 50
            var guard = 1.0
            var itemA = entities[i]
            for(var j = i + 1; j < entities.length; j++) {
                var itemB = entities[j]
                var totalMinDistance = Math.max(itemA.height, itemA.width) / 2.0
                        + Math.max(itemB.height, itemB.width) / 2.0
                        + minDistance
                var centerA = itemCenter(itemA)
                var centerB = itemCenter(itemB)
                var xDiff = centerA.x - centerB.x
                var yDiff = centerA.y - centerB.y
                var length = Math.sqrt(xDiff*xDiff + yDiff*yDiff)
                if(length < guard) {
                    continue
                }
                var lengthDiff = length - totalMinDistance
                if(lengthDiff > 0.0) {
                    continue
                }

                var xDelta = lengthDiff * xDiff / length
                var yDelta = lengthDiff * yDiff / length
                var k = neuronifyRoot.width * 0.007
                if(!itemA.dragging) {
                    itemA.velocity.x -= 0.5 * k * xDelta
                    itemA.velocity.y -= 0.5 * k * yDelta
                }
                if(!itemB.dragging) {
                    itemB.velocity.x += 0.5 * k * xDelta
                    itemB.velocity.y += 0.5 * k * yDelta
                }
            }
        }

        var maxAppliedSpeed = 0.0
        var maxSpeed = neuronifyRoot.width * 1.0
        var minSpeed = neuronifyRoot.width * 0.5
        for(var i in entities) {
            var item = entities[i]
            var speed = Math.sqrt(item.velocity.x*item.velocity.x + item.velocity.y*item.velocity.y)
            if(speed > maxSpeed && speed > 0) {
                item.velocity.x *= (maxSpeed / speed)
                item.velocity.y *= (maxSpeed / speed)
            }

            maxAppliedSpeed = Math.max(maxAppliedSpeed, item.velocity.x*item.velocity.x + item.velocity.y*item.velocity.y)
            item.x += item.velocity.x * dt
            item.y += item.velocity.y * dt

            item.x = Math.max(item.x, - item.width * 0.5)
            item.y = Math.max(item.y, - item.height * 0.5)
            item.x = Math.min(item.x, neuronLayer.width - item.width * 0.5)
            item.y = Math.min(item.y, neuronLayer.height - item.height  * 0.5)
        }

        if(maxAppliedSpeed < minSpeed && !anyDragging) {
            layoutTimer.stop()
        }

        lastOrganizeTime = currentOrganizeTime
    }

    function resetOrganize() {
        lastOrganizeTime = Date.now()
        layoutTimer.start()
    }

    Timer {
        id: layoutTimer
        interval: 24
        running: true
        repeat: true
        onTriggered: {
            organize()
        }
    }
}

