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
import "qrc:/qml/style"


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
    property alias plotKernel: retinaEngine.plotKernel


    color: "#0088aa"
    width: 240
    height: 180
    canReceiveConnections: false
    controls: controlsComponent

    engine: RetinaEngine {
        id: retinaEngine
        kernel: kernel
        videoSurface: root.videoSurface
        plotKernel: true
    }

    savedProperties: [
        PropertyGroup {
            property alias x: root.x
            property alias y: root.y
            property alias kernelType: root.kernelType
            property alias sensitivity: root.sensitivity
            property alias plotKernel: root.plotKernel
        },
        PropertyGroup { // Note: Don't reorder! This needs to be saved after kernelType.
            property alias kernelProperties: kernelLoader.item
        }
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
        abstractKernelEngineType: kernelLoader.item ?
                                      kernelLoader.item : null

        imageAlpha: 225
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
        curveColor: "#0088aa"
        connectorColor: "#0088aa"
    }

    Component {
        id: controlsComponent
        PropertiesPage {
            property string title: "Retina"
            Component.onCompleted: {
                for(var i = 0; i < fieldTypes.count; i++) {
                    var item = fieldTypes.get(i)
                    if(Qt.resolvedUrl(item.value) ===
                            Qt.resolvedUrl(root.kernelType)) {
                        fieldTypesView.currentIndex = i
                        break
                    }
                }
            }

            spacing: 10
            Text {
                text: "Receptive Field: " +
                      fieldTypes.get(fieldTypesView.currentIndex).name

                font: Style.control.font
                color: Style.text.color
            }

            GridView {
                id: fieldTypesView
                anchors {
                    left: parent.left
                    right: parent.right
                }

                cellWidth: Style.touchableSize
                cellHeight: Style.touchableSize
                height: Style.touchableSize

                property bool created: false

                onChildrenChanged: {
                    if(!currentIndex+1){
                        currentIndex = fieldIndex
                    }
                }
                onCurrentIndexChanged: {
                    if(created){
                        kernelType = model.get(currentIndex).value
                        fieldIndex = currentIndex
                    }else{
                        created = true
                        for(var i = 0; i < fieldTypes.count; i++) {
                            var item = fieldTypes.get(i)
                            if(Qt.resolvedUrl(item.name) ===
                                    Qt.resolvedUrl(root.kernelType)) {
                                fieldTypesView.currentIndex = i
                                break
                            }
                        }
                    }
                }
                interactive: false
                model: ListModel {
                    id: fieldTypes
                    ListElement {
                        name: "Orientation selective"
                        key: "qrc:/images/sensors/kernels/gabor.png"
                        value: "kernels/GaborKernel.qml"
                    }
                    ListElement {
                        name: "Center-surround"
                        key: "qrc:/images/sensors/kernels/dog.png"
                        value: "kernels/DogKernel.qml"
                    }
                    ListElement {
                        name: "Rectangular"
                        key: "qrc:/images/sensors/kernels/offTop.png"
                        value: "kernels/RectangularKernel.qml"
                    }
                }
                clip: true
                delegate: Item {
                    height: Style.touchableSize
                    width: height

                    Image {
                        id: fieldTypesImage
                        anchors.fill: parent
                        source: model.key
                    }
                    MouseArea {
                        anchors.fill: parent
                        onClicked: {
                            fieldTypesView.currentIndex = index
                        }
                    }
                }
                highlight: Rectangle {
                    color: "#deebf7"
                }

            }

            CheckBox {
                Text{
                    anchors.left: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Show receptive field"
                    font: Style.control.font
                    color: Style.text.color
                }

                checked: retinaEngine.plotKernel
                onCheckedChanged: {
                    if(checked) {
                        retinaEngine.plotKernel = true
                    }else{
                        retinaEngine.plotKernel = false
                    }
                }
            }


            //Slider to change the sensitivity:
            BoundSlider {
                minimumValue: 0
                maximumValue: 10e3
                stepSize: 1e2
                target: retinaEngine
                property: "sensitivity"
                text: "Sensitivity"
            }


            Loader{
                anchors {
                    left: parent.left
                    right: parent.right
                }
                sourceComponent: (kernelLoader.item && kernelLoader.item.controls) ? kernelLoader.item.controls : undefined
            }

        }


    }



}


