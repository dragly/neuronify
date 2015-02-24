import QtQuick 2.0
import QtQuick.Dialogs 1.2
import QtQuick.Window 2.0
import Neuronify 1.0

Item {
    property Item neuronify
    function saveState(fileUrl) {
        var neurons = neuronify.neurons
        var sensors = neuronify.sensors
        var voltmeters = neuronify.voltmeters
        if (!String.format) {
          String.format = function(format) {
            var args = Array.prototype.slice.call(arguments, 1);
            return format.replace(/{(\d+)}/g, function(match, number) {
              return typeof args[number] != 'undefined'
                ? args[number]
                : match
              ;
            });
          };
        }

        var fileString = ""

        console.log("Saving to " + fileUrl)

        var counter = 0
        for(var i in neurons) {
            var neuron = neurons[i]
            console.log(neuron.x)
            var ss = "var neuron{0} = createNeuron({x: {1}, y: {2}, clampCurrent: {3}, clampCurrentEnabled: {4}, adaptationIncreaseOnFire: {5}, outputStimulation: {6}})"
            ss = String.format(ss,i.toString(),neuron.x, neuron.y, neuron.clampCurrent,
              neuron.clampCurrentEnabled, neuron.adaptationIncreaseOnFire, neuron.outputStimulation)
            console.log(ss)
            fileString += ss + "\n"
        }

        for(var i in neurons) {
            var neuron = neurons[i]
            for(var j in neuron.connections){
                var toNeuron = neuron.connections[j].itemB
                var indexOfToNeuron = neurons.indexOf(toNeuron)
                fileString += String.format("connectNeurons(neuron{0}, neuron{1}) \n",i,indexOfToNeuron)
            }
        }

        for(var i in voltmeters){
            var voltmeter = voltmeters[i]
            fileString += String.format("var voltmeter{0} = createVoltmeter({x: {1}, y:{2}}) \n", i, voltmeter.x, voltmeter.y)
            var neuronIndex = neurons.indexOf(voltmeter.connectionPlots[0].connection.itemA)
            fileString += String.format("connectVoltmeterToNeuron(neuron{0}, voltmeter{1}) \n",neuronIndex, i)
        }

        for(var i in sensors) {
            fileString += sensors[i].dump(i, neurons)
        }


        saveFileIO.source = fileUrl
        saveFileIO.write(fileString)
    }

    function read(fileUrl) {
        console.log("Reading file " + fileUrl)
        creationControls.autoLayout = false
        deleteEverything()
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

