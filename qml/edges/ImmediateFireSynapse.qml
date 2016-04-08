import QtQuick 2.0
import Neuronify 1.0

import ".."
import "../controls"
import "../hud"

Edge {
    objectName: "ImmediateFireSynapse"
    filename: "synapses/ImmediateFireSynapse.qml"

    engine: EdgeEngine {
        id: engine

        onStepped:{
            currentOutput = 0.0;
        }

        onReceivedFire: {
            currentOutput = 1e6;
        }
    }
}
