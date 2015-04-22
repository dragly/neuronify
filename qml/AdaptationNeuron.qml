import QtQuick 2.0
import Neuronify 1.0

BaseNeuronModel {
    Conductance {
        id: adaptationConductance
        conductance: 0.0
        onStep: {
            console.log("On step!")
            conductance = -conductance * dt
        }
        onFire: {
            console.log("On fire!")
            conductance += 1.0
        }
    }
}

