import QtQuick 2.0

Column {
    id: viewColumn
    property int currentIndex

    Component.onCompleted: {
        for(var i in children) {
            var child = children[i]
            child.index = i
        }
    }
}
