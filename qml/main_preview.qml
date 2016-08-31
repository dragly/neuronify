import QtQuick 2.0

Item {
    id: root
    width: 640
    height: 480

    property var objects: []

    function load(contents, filename) {
        for(var i in objects) {
            var oldObject = objects[i]
            console.log("Destroy", oldObject)
            oldObject.destroy()
        }
        objects = []

        var object = Qt.createQmlObject(contents, root, filename)
        object.parent = root
        root.width = object.width
        root.height = object.height

        objects.push(object)
    }
}
