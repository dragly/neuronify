import QtQuick 2.0
import QtQuick.Controls 1.3
import QtMultimedia 5.4
import Neuronify 1.0
import "../paths"
import "../hud"
import "../controls"
import ".."
import "../paths"
import "../hud"
import "../controls"
import ".."


/*!
\qmltype Retina
\inqmlmodule Neuronify
\ingroup neuronify-sensors
\brief Visual sensor that can be connected to neurons to generate activity
based on the receptive field of the sensor and the visual stimuli captured by
the camera.
*/

Node {
    id: root
    objectName: "retina"
    fileName: "sensors/Retina.qml"

    property point connectionPoint: Qt.point(x + width / 2, y + height / 2)
    property VideoSurface videoSurface: null;
    property int fieldIndex: 0
    property int viewIndex: 0

    width: 240
    height: 180

    dumpableProperties: [
        "x",
        "y"
    ]

    onVideoSurfaceChanged: {
        if(!videoSurface){
            return
        }
        if(!videoSurface.camera.ActiveState){
            videoSurface.camera.start()
        }
    }

    ReceptiveField{
        id:recField
        resolutionHeight : 50
        resolutionWidth : 50
        spatialType: ReceptiveField.OffLeftRF
    }

    engine: RetinaEngine {
        id: retinaEngine
        receptiveField: recField
        videoSurface: root.videoSurface
        plotReceptiveField: false
    }

    controls: Component {
        Column {
            anchors.fill: parent

            Text {
                text: "Resolution Height: " + recField.resolutionHeight.toFixed(1)
            }
            BoundSlider {
                minimumValue: 10
                maximumValue: 300
                stepSize: 1
                target: recField
                property: "resolutionHeight"
            }

            Text {
                text: "Resolution Width: " + recField.resolutionWidth.toFixed(1)
            }
            BoundSlider {
                minimumValue: 10
                maximumValue: 300
                stepSize: 1
                target: recField
                property: "resolutionWidth"
            }
            Text {
                text: "Receptive Field: "
            }
            ComboBox {
                id: comboBox
                width: 200
                model: fieldTypes

                onChildrenChanged: {
                    if(!currentIndex+1){
                        currentIndex = fieldIndex
                    }
                }

                onCurrentIndexChanged: {
                    recField.spatialType = model.get(currentIndex).name
                    fieldIndex = currentIndex
                }

            }
            ComboBox {
                id: displayComboBox
                width: 200
                model: imageView

                onChildrenChanged: {
                    if(!currentIndex+1){

                        currentIndex = viewIndex
                    }
                }

                onCurrentIndexChanged: {
                    //recField.receptiveFieldType = model.get(currentIndex).name
                    if(currentIndex == 0) retinaEngine.plotReceptiveField = false
                    else retinaEngine.plotReceptiveField = true
                    viewIndex = currentIndex
                }

            }

        }

    }

    ListModel {
        id: fieldTypes
        ListElement {text: "Gabor"; name: ReceptiveField.GaborRF}
        ListElement {text: "Off-left";   name: ReceptiveField.OffLeftRF}
        ListElement {text: "Off-right";  name: ReceptiveField.OffRightRF}
        ListElement {text: "Off-top";    name: ReceptiveField.OffTopRF}
        ListElement {text: "Off-bottom"; name: ReceptiveField.OffBottomRF}

    }


    ListModel {
        id: imageView
        ListElement {text: "Camera";   name: 1}
        ListElement {text: "Receptive Field";   name: 2}
    }

    Rectangle {
        color: "#756bb1"
        anchors.fill: parent
        radius: 5
        border.width: 2.0
        border.color: "#BCBDDC"
    }

    RetinaPainter {
        id: retinaPainter
        retinaEngine: retinaEngine
        anchors {
            fill: parent
            margins: 5
        }
        property bool plotReceptiveField: true
    }

    ResizeRectangle {
    }

    Connector {
        visible: root.selected
        onDropped: {
            root.droppedConnector(root, connector)
        }
    }
}


