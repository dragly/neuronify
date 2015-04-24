import QtQuick 2.0
import "../style"

Row {
    Component.onCompleted: {
        for(var i in children) {
            var child = children[i]
            if(child.objectName === "CreationItem") {
                child.dropped.connect(droppedEntity)
            }
        }
    }
    spacing: Style.touchableSize * 0.5
}

