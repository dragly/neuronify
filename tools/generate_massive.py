from pylab import *
import json

nodes = [
    {
        "fileName": "generators/PoissonGenerator.qml",
        "label": "",
        "x": 100,
        "y": 100,
        "engine": {
            "fireOutput": 0.0001,
            "rate": 300
        }
    },
    # {
    #     "fileName":"meters/RasterPlot.qml",
    #     "label":"",
    #     "x":96,
    #     "y":-400,
    #     "height":800,
    #     "showLegend":True,
    #     "timeRange":0.10000000000000002,
    #     "width":1200
    # },
    {
        "fileName":"generators/ACClamp.qml",
        "label":"",
        "x":0,
        "y":0,
        "engine":{
            "amplitude":0.00001,
            "frequency":40,
            "time":0.10300000000000192
        }
    },
    {"fileName":"meters/Speaker.qml","label":"","x":267.7179788505151,"y":0.1856258160838,"source":"glass.wav"},
]

edges = [{
    "from": 0,
    "to": 1
}]

column_count = 64
offset_count = len(nodes)
neuron_count = 512

poisson_index = 0
ac_index = 1
speaker_index = 2

for i in range(neuron_count):
    # poisson_index = offset_count + 2 * i
    neuron_index = offset_count + i
    
    # if not (i % neuron_count / 100):
    #     edge_neuron_main = {
    #         "from": neuron_index,
    #         "to": 1
    #     }
    #     edges.append(edge_neuron_main)
    
    edge_poisson_neuron = {
        "from": poisson_index,
        "to": neuron_index
    }
    edge_ac_neuron = {
        "from": ac_index,
        "to": neuron_index
    }
    edge_speaker_neuron = {
        "from": neuron_index,
        "to": speaker_index
    }
    
    edges.append(edge_poisson_neuron)
    edges.append(edge_ac_neuron)
    edges.append(edge_speaker_neuron)
    
    factor = 1.0
    if rand() < 0.5:
        factor = -1.0
    fire_output = factor * 0.00001
    
    neuron = {
        "fileName": "neurons/PassiveNeuron.qml",
        "label": "",
        "x": 100 + 64 * (i % column_count),
        "y": 500 + 200 * floor(i / column_count),
        "engine": {
            "capacitance": 0.000001001,
            "fireOutput": fire_output,
            "initialPotential": -0.08,
            "restingPotential": -0.065,
            "synapticConductance": 0,
            "synapticPotential": 0.04999999999999999,
            "synapticTimeConstant": 0.01,
            "threshold": (2.0 * rand() - 1.0) * 30e-3,
            "voltage": -0.0763497668807441
        },
        "resistance": 10000,
    }
    
    # nodes.append(poisson)
    nodes.append(neuron)
    
    
data = {
    "edges": edges,
    "nodes": nodes,
    "fileFormatVersion": 2,
    "workspace": {
        "playbackSpeed": 2,
        "visibleRectangle": {
            "height": 778.8299070684317,
            "width": 1372.8527175443542,
            "x": -177.09803541776617,
            "y": 179.8000536767801
        }
    }
}    

latest_file = open("/home/svenni/.config/Ovilab/Neuronify/latest.nfy", "w")
latest_file.write(json.dumps(data))
