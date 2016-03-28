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
    property var workspace: null
    property var otherItems: []

    function showSaveDialog() {
        saveFileDialog.visible = true
    }

    function showLoadDialog() {
        loadFileDialog.visible = true
    }

    function saveState(fileUrl) {
        var nodes = graphEngine.nodes
        var edges = graphEngine.edges
        var nodeList = [];
        var edgeList = [];
        var otherList = [];

        console.log("Saving to " + fileUrl)

        var counter = 0
        for(var i in nodes) {
            var entity = nodes[i];
            var dump = entity.dump(i);
            if(dump) {
                nodeList.push(dump);
            }
        }

        for(var i in edges) {
            var connection = edges[i]
            edgeList.push(connection.dump(i, graphEngine))
        }

        var workspaceProperties = {
            x: workspace.x,
            y: workspace.y,
            scale: workspace.scale,
            playbackSpeed: workspace.playbackSpeed,
        };

        var result = {
            edges: edgeList,
            nodes: nodeList,
            workspace: workspaceProperties
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

