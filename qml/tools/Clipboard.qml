import QtQuick 2.0

Rectangle {
    function copyNeurons() {
        console.warn("Copy paste not implemented!")
        return

        copiedNeurons = []
        var copiedNeuron = []
        for(var i in selectedEntities) {
            var neuron = selectedEntities[i]
            copiedNeuron = neuron
            copiedNeurons.push(copiedNeuron)
        }
        selectedEntities = []
    }

    function pasteNeurons() {
        console.warn("Copy paste not implemented!")
        return

        var newNeurons = []
        for(var i in copiedNeurons) {
            var neuronToCopy = copiedNeurons[i]
            var neuron = createNeuron({
                                          x: neuronToCopy.x + 10,
                                          y: neuronToCopy.y + 10,
                                          copiedFrom: neuronToCopy
                                      })

            newNeurons.push(neuron)
        }
        for(var i in copiedNeurons) {
            var oldNeuron = copiedNeurons[i]
            for(var j in newNeurons) {
                var newNeuron = newNeurons[j]
                if(newNeuron.copiedFrom === oldNeuron) {
                    // Find connections
                    for(var k in oldNeuron.connections) {
                        var connectedToNeuron = oldNeuron.connections[k].itemB
                        // Check if connected to copied neuron
                        for(var l in copiedNeurons) {
                            var otherNeuron = copiedNeurons[l]
                            if(otherNeuron === connectedToNeuron) {
                                // Find copied twin
                                for(var m in newNeurons) {
                                    var otherNewNeuron = newNeurons[m]
                                    if(otherNewNeuron.copiedFrom === otherNeuron) {
                                        connectNeurons(newNeuron, otherNewNeuron)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        resetOrganize()
    }
}

