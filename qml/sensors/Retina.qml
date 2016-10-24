import QtQuick 2.6
import QtQuick.Controls 1.3
import QtMultimedia 5.4
import QtQuick.Window 2.0

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
import "../edges"


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
    filename: "sensors/Retina.qml"
    square: true

    property point connectionPoint: Qt.point(x + width / 2, y + height / 2)
    property VideoSurface videoSurface: null;
    property int fieldIndex: 0
    property int viewIndex: 0
    property string kernelType: "kernels/GaborKernel.qml"
    property alias sensitivity: retinaEngine.sensitivity
    property alias plotKernel: retinaEngine.plotKernel
    readonly property real instantRate: retinaEngine.instantRate
    property bool negativeRate: instantRate < 0

    property bool cameraAvailable: QtMultimedia.availableCameras.length > 0

    preferredEdge: CurrentSynapse {}
    color: "#dd5000"
    width: 240
    height: 240
    canReceiveConnections: false
    controls: controlsComponent

    engine: RetinaEngine {
        id: retinaEngine
        kernel: kernel
        videoSurface: root.videoSurface
        plotKernel: cameraAvailable
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
        id: backgroundRect
        color: "#ffcc00"
        anchors.fill: parent
        radius: 5
        border.width: 5.0
        border.color: "#ffcc00"
    }



    Rectangle {
        id: rateBarBackground
        anchors {
            top: backgroundRect.bottom
        }
        radius: 5
        border.width: 10.0
        border.color: "#ffcc00"
        color: "#ffcc00"
        height: parent.height * 0.1
        width: parent.width
    }

    Rectangle {
        id: rateBar
        anchors {
            verticalCenter:  rateBarBackground.verticalCenter
        }
        radius: rateBarBackground.radius
        color: "#ffcc00"
        height: parent.height * 0.08
        width: parent.width * Math.min(Math.abs(instantRate), 0.49)
    }


    //    Rectangle {
    //        id: zeroMark
    //        anchors {
    //            centerIn: rateBarBackground
    //        }
    //        radius: 5
    //        color: "#e41a1c"
    //        height: rateBar.height
    //        width: rateBarBackground.width * 0.02
    //        opacity: 1.0
    //    }

    onNegativeRateChanged: {
        if(negativeRate){
            rateBar.anchors.right =  rateBarBackground.horizontalCenter
            rateBar.anchors.left =  undefined
            rateBar.color =  Qt.rgba(0.5, 0.5, 0.5);
        }else{
            rateBar.anchors.left =  rateBarBackground.horizontalCenter
            rateBar.anchors.right =  undefined
            rateBar.color =  "#dd5000";
        }

    }


    RetinaPainter {
        id: retinaPainter
        property list<Rotation> windowsTransforms: [
            Rotation {
                axis.x:1
                axis.y:0
                axis.z:0
                origin.x: retinaPainter.width * 0.5
                origin.y: retinaPainter.height * 0.5
                angle: 180
            },
            Rotation {
                axis.y:1
                axis.x: 0
                axis.z: 0
                origin.x: retinaPainter.width * 0.5
                origin.y: retinaPainter.height * 0.5
                angle: 180
            }
        ]
        property list<Rotation> osxTransforms: [
            Rotation {
                axis.x: 0
                axis.y: 1
                axis.z: 0
                origin.x: retinaPainter.width * 0.5
                origin.y: retinaPainter.height * 0.5
                angle: 180
            }
        ]
        property list<Rotation> iosTransforms: [
            Rotation {
                axis.x: 0
                axis.y: 0
                axis.z: 1
                origin.x: retinaPainter.width * 0.5
                origin.y: retinaPainter.height * 0.5
                angle: {
                    switch(Screen.primaryOrientation) {
                    case Qt.PortraitOrientation:
                        return 90;
                    case Qt.LandscapeOrientation:
                        return 0;
                    case Qt.InvertedPortraitOrientation:
                        return 270;
                    case Qt.LandscapeOrientation:
                        return 180;
                    }
                }
            }
        ]
        property list<Rotation> linuxTransforms: [
            Rotation {
                axis.x: 0
                axis.y: 1
                axis.z: 0
                origin.x: retinaPainter.width * 0.5
                origin.y: retinaPainter.height * 0.5
                angle: 180
            }
        ]
        property list<Rotation> androidTransforms: [
            Rotation {
                axis.x: 0
                axis.y: 0
                axis.z: 1
                origin.x: retinaPainter.width * 0.5
                origin.y: retinaPainter.height * 0.5
                angle: {
                    switch(Screen.primaryOrientation) {
                    case Qt.PortraitOrientation:
                        return 90;
                    case Qt.LandscapeOrientation:
                        return 0;
                    case Qt.InvertedPortraitOrientation:
                        return 270;
                    case Qt.LandscapeOrientation:
                        return 180;
                    }
                }
            }
        ]
        property list<Rotation> noTransforms
        property bool plotKernel: true
        anchors {
            fill: parent
            margins: 5
        }

        retinaEngine: retinaEngine

        transform: {
            switch(Qt.platform.os) {
            case "windows":
                return windowsTransforms;
            case "osx":
                return osxTransforms;
            case "linux":
                return linuxTransforms;
            case "android":
                return androidTransforms;
            case "ios":
                return iosTransforms;
            }
            return noTransforms;
        }
    }

    ResizeRectangle {
        forceSquare: true
    }

    Connector {
        visible: root.selected
        color: "#dd5000"
        connectorColor: "#dd5000"
        initialPoint: Qt.point(root.width + 32, root.height + 32)
        attachmentPoint: Qt.point(root.width, root.height)
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

    Rectangle {
        anchors {
            fill: cameraMissingText
            margins: -8
        }
        visible: cameraMissingText.visible
    }

    Text {
        id: cameraMissingText
        anchors.centerIn: parent
        visible: !cameraAvailable
        font.pixelSize: 20
        wrapMode: Text.WrapAtWordBoundaryOrAnywhere
        clip: true
        text: "No camera found"
    }
}


