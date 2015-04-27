import QtQuick 2.0
import "../style"

Row {
    signal droppedEntity(var fileUrl, var properties, var controlParent, var useAutoLayout)
    signal clicked(var entity)

    Component.onCompleted: {
        for(var i in children) {
            var child = children[i]
            if(child.objectName === "CreationItem") {
                child.dropped.connect(droppedEntity)
                child.clicked.connect(clicked)
            }
        }
    }
    spacing: Style.touchableSize * 0.5
}

