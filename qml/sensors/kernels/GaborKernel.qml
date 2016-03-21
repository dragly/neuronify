import QtQuick 2.0
import Neuronify 1.0


Item {

    property alias resolutionWidth: engine.resolutionWidth
    property alias resolutionHeight: engine.resolutionHeight
    property alias engine: engine

    GaborKernelEngine{
        id: engine
        resolutionHeight: 80
        resolutionWidth: 80
    }



}
