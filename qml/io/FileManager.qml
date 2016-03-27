import QtQuick 2.0
import QtQuick.Dialogs 1.1
import QtQuick.Window 2.0
import Neuronify 1.0

/*!
\qmltype FileManager
\inqmlmodule Neuronify
\ingroup neuronify-io
\brief A type to contain the save/load features

Neuronify saves configurations by storing the javascript code needed to create
the network, and loads files by evaluating the generated and saved js code.

The read/write part is done in \l{FileIO}, while this object just treats the file
as a js string.
*/

Item {
    signal loadState(var fileUrl)

    property GraphEngine graphEngine: null
    property var otherItems: []

    function showSaveDialog() {
        saveFileDialog.visible = true
    }

    function showLoadDialog() {
        loadFileDialog.visible = true
    }

    function saveState(fileUrl) {
//        console.warn("WARNING: Save disabled!");
//        return;
        var entities = graphEngine.nodes
        var connections = graphEngine.edges
        var entityList = [];
        var connectionList = [];
        var otherList = [];
        console.log("Saving to " + fileUrl)

        var counter = 0
        for(var i in entities) {
            var entity = entities[i];
            var dump = entity.dump(i);
            if(dump) {
                entityList.push(dump);
            }
        }

        for(var i in connections) {
            var connection = connections[i]
            connectionList.push(connection.dump(i, graphEngine))
        }

//        for(var i in otherItems) {
//            var item = otherItems[i]
//            otherList.push(item.dump())
//        }

        var result = {
            connections: connectionList,
            entities: entityList,
            other: otherList
        };
        var fileString = JSON.stringify(result);

        console.log(fileString)

        saveFileIO.source = fileUrl
        saveFileIO.write(fileString)
    }

    function read(fileUrl) {
        console.log("Reading file " + fileUrl)
        if(!fileUrl) {
            loadFileIO.source = "";
        } else {
            loadFileIO.source = fileUrl
        }
        var stateFile = loadFileIO.read()
        return stateFile
    }

    FileIO {
        id: loadFileIO
        source: "none"
        onError: console.log(msg)
    }

    FileIO {
        id: saveFileIO
        source: "none"
        onError: console.log(msg)
    }

    FileDialog {
        id: saveFileDialog
        title: "Please enter a filename"
        visible : false
        selectExisting: false
        nameFilters: Qt.platform.os === "osx" ? [] : ["Neuronify files (*.nfy)", "All files (*)"]

        onAccepted: {
            saveState(fileUrl)
        }
    }

    FileDialog {
        id: loadFileDialog
        title: "Please choose a file"
        visible : false
        nameFilters: Qt.platform.os === "osx" ? [] : ["Neuronify files (*.nfy)", "All files (*)"]

        onAccepted: {
            console.log("Load dialog accepted")
            loadState(fileUrl)
        }
    }
}

