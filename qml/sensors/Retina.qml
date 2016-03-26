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
    square: true


    property point connectionPoint: Qt.point(x + width / 2, y + height / 2)
    property VideoSurface videoSurface: null;
    property int fieldIndex: 0
    property int viewIndex: 0
    property string kernelType: "kernels/GaborKernel.qml"
    property alias sensitivity: retinaEngine.sensitivity

    width: 240
    height: 180

    dumpableProperties: [
        "x",
        "y",
        "kernelType",
        "sensitivity"
    ]

    onVideoSurfaceChanged: {
        if(!videoSurface){
            return
        }
        if(!videoSurface.camera.ActiveState){
            videoSurface.camera.start()
        }
    }

    Loader{
        id: kernelLoader
        source: root.kernelType
    }

    Kernel{
        id:kernel
        resolutionHeight : kernelLoader.item ?
                               kernelLoader.item.resolutionHeight: 80
        resolutionWidth : kernelLoader.item ?
                              kernelLoader.item.resolutionWidth : 80
        abstractKernelEngineType: kernelLoader.item ?
                                      kernelLoader.item.engine : null
    }

    engine: RetinaEngine {
        id: retinaEngine
        kernel: kernel
        videoSurface: root.videoSurface
        plotKernel: false
    }

    controls: Component {
        Column {
            anchors.fill: parent


            // Slider to change the resolution:
            //            Text {
            //                text: "Resolution Height: " + kernel.resolutionHeight.toFixed(1)
            //            }
            //            BoundSlider {
            //                minimumValue: 10
            //                maximumValue: 300
            //                stepSize: 100
            //                target: kernel
            //                property: "resolutionHeight"
            //            }

            //            Text {
            //                text: "Resolution Width: " + kernel.resolutionWidth.toFixed(1)
            //            }
            //            BoundSlider {
            //                minimumValue: 10
            //                maximumValue: 300
            //                stepSize: 100
            //                target: kernel
            //                property: "resolutionWidth"
            //            }


            Component.onCompleted: {
                for(var i = 0; i < fieldTypes.count; i++) {
                    var item = fieldTypes.get(i)
                    if(Qt.resolvedUrl(item.name) ===
                            Qt.resolvedUrl(root.kernelType)) {
                        comboBox.currentIndex = i
                        break
                    }
                }
            }

            Text {
                text: "Show: "
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
                    if(currentIndex == 0) retinaEngine.plotKernel = false
                    else retinaEngine.plotKernel = true
                    viewIndex = currentIndex
                }

            }
            Text {
                text: "Receptive Field: "
            }
            ComboBox {
                id: comboBox
                width: 200
                model: fieldTypes
                property bool created: false

                onChildrenChanged: {
                    if(!currentIndex+1){
                        currentIndex = fieldIndex
                    }
                }

                onCurrentIndexChanged: {
                    if(created){
                        kernelType = model.get(currentIndex).name
                        fieldIndex = currentIndex
                    }else{
                        created = true
                        for(var i = 0; i < fieldTypes.count; i++) {
                            var item = fieldTypes.get(i)
                            if(Qt.resolvedUrl(item.name) ===
                                    Qt.resolvedUrl(root.kernelType)) {
                                comboBox.currentIndex = i
                                break
                            }
                        }
                    }
                }

            }

            //Slider to change the sensitivity:
            Text {
                text: "Sensitivity: " + retinaEngine.sensitivity.toFixed(0)
            }
            BoundSlider {
                minimumValue: 1
                maximumValue: 10
                stepSize: 1
                target: retinaEngine
                property: "sensitivity"
            }

        }

    }

    ListModel {
        id: fieldTypes
        ListElement {text: "Orientation selective";
            name: "kernels/GaborKernel.qml"}
        ListElement {text: "Center-surround"; name: "kernels/DogKernel.qml"}
        ListElement {text: "OffLeft"; name: "kernels/OffLeftKernel.qml"}
        ListElement {text: "OffRight"; name: "kernels/OffRightKernel.qml"}
        ListElement {text: "OffTop"; name: "kernels/OffTopKernel.qml"}
        ListElement {text: "OffBottom"; name: "kernels/OffBottomKernel.qml"}
    }


    ListModel {
        id: imageView
        ListElement {text: "Camera";   name: 1}
        ListElement {text: "Receptive Field";   name: 2}
    }

    Rectangle {
        color: "#0088aa"
        anchors.fill: parent
        radius: 5
        border.width: 0.0
        border.color: "#80e5ff"
    }

    RetinaPainter {
        id: retinaPainter
        retinaEngine: retinaEngine
        anchors {
            fill: parent
            margins: 5
        }
        property bool plotKernel: true
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


