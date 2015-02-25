import QtQuick 2.0
import QtQuick.Dialogs 1.2
import QtQuick.Window 2.0
import Neuronify 1.0

Item {
    signal loadState(var fileUrl)

    property var entities: []
    property var connections: []

    function showSaveDialog() {
        saveFileDialog.visible = true
    }

    function showLoadDialog() {
        loadFileDialog.visible = true
    }

    function saveState(fileUrl) {
        var fileString = ""
        console.log("Saving to " + fileUrl)

        var counter = 0
        for(var i in entities) {
            var entity = entities[i]
            fileString += entity.dump(i)
        }

        for(var i in connections) {
            var connection = connections[i]
            fileString += connection.dump(i, entities)
        }

        console.log(fileString)

        saveFileIO.source = fileUrl
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
        nameFilters: ["Neuronify files (*.nfy)", "All files (*)"]

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
        nameFilters: ["Neuronify files (*.nfy)", "All files (*)"]

        onAccepted: {
            loadState(fileUrl)
        }
    }
}

