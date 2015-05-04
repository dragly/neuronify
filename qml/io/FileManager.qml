import QtQuick 2.0
import QtQuick.Dialogs 1.1
import QtQuick.Window 2.0
import Neuronify 1.0

/*!
\qmltype FileManager
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
        var entities = graphEngine.nodes
        var connections = graphEngine.edges
        var fileString = ""
        console.log("Saving to " + fileUrl)

        var counter = 0
        for(var i in entities) {
            var entity = entities[i]
            fileString += entity.dump(i)
        }

        for(var i in connections) {
            var connection = connections[i]
            fileString += connection.dump(i, graphEngine)
        }

        for(var i in otherItems) {
            var item = otherItems[i]
            fileString += item.dump()
        }

        console.log(fileString)

        save

        .source = fileUrl
        saveFileIO.write(fileString)
    }

    function read(fileUrl) {
        console.log("Reading file " + fileUrl)
        loadFileIO.source = fileUrl
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
        nameFilters: Qt.platform.os === "osx" ? [] : ["Nestify files (*.nfy)", "All files (*)"]

        onAccepted: {
            var fileUrlNew = fileUrl
            var extensionSplit = fileUrlNew.toString().split(".")
            var fileExtension = extensionSplit[extensionSplit.length - 1]
            if(fileExtension !== "nfy") {
                fileUrlNew = Qt.resolvedUrl(fileUrlNew.toString() + ".nfy")
            }
            saveState(fileUrlNew)
        }
    }

    FileDialog {
        id: loadFileDialog
        title: "Please choose a file"
        visible : false
        nameFilters: Qt.platform.os === "osx" ? [] : ["Nestify files (*.nfy)", "All files (*)"]

        onAccepted: {
            console.log("Load dialog accepted")
            loadState(fileUrl)
        }
    }
}

