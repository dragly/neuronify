pragma Singleton
import QtQuick 2.0

Item {
    id: root
    property string device: "desktop"

    property real windowWidth
    property real windowHeight
    property real minimumTouchableSize: windowWidth / 25
    property real maximumTouchableSize: windowWidth / 10
    property real pixelDensity: 72
    property real touchableSize: 6 * size
    property real size: 72
    property real margin: 4 * size
    property real baseMargin: margin
    property real scale: 1.0

    property alias font: fontObject
    property alias button: buttonObject

    Item {
        id: buttonObject
        property color color: Qt.rgba(0.15, 0.15, 0.15, 1.0)
        property color backgroundColor: "#dedede"
        property alias font: fontProxy.font
        Text {
            id: fontProxy
            font.pixelSize: 3 * root.size
            font.weight: Font.Light
            font.family: "Roboto"
        }
    }

    Item {
        id: fontObject
        property alias heading: headingObject

        property real size: 2.5 * root.size
        property color color: Qt.rgba(0.15, 0.15, 0.15, 0.9)
        property int weight: Font.Light
        property string family: "Roboto"

        Item {
            id: headingObject
            property color color: Qt.rgba(0.4, 0.4, 0.4, 1.0)
            property real size: 2.5 * font.size
        }
    }

    function reset(width, height, pixelDensity) {
        root.windowWidth = width
        root.windowHeight = height
        root.pixelDensity = pixelDensity

        if(Qt.platform.os === "android" || Qt.platform.os === "ios") {
            if(pixelDensity === 0) {
                console.warn("Style.reset(): Pixel density is zero. Assuming 4 ppmm.")
                pixelDensity = 4
            }

            var deviceWidth = width / pixelDensity
            var deviceHeight = height / pixelDensity

            size = pixelDensity

            scale = pixelDensity / 4

            if(deviceWidth > 160 && deviceHeight > 100) {
                device = "tablet"
            } else {
                device = "phone"
            }
        } else {
            device = "dekstop"
            size = root.windowWidth * 0.01
        }
    }
}
