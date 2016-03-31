pragma Singleton
import QtQuick 2.6
import QtQuick.Controls 1.0

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

    property alias text: textObject
    property alias heading: headingObject
    property alias font: textFontProxy.font
    property alias button: buttonObject
    property alias control: controlObject

    property alias color: colorsObject
    property alias border: borderObject

    QtObject {
        id: colorsObject
        property color background: "#f7fbff"
        property color foreground: "#deebf7"
        property alias border: borderObject.color
    }

    QtObject {
        id: borderObject
        property color color: "#9ecae1"
        property color lightColor: "#d5e8f2"
        property real width: 2.0
    }

    Item {
        id: buttonObject
        property alias font: fontProxy.font
        property color color: Qt.rgba(0.15, 0.15, 0.15, 1.0)
        property color backgroundColor: "#dedede"
        Text {
            id: fontProxy
            font.pixelSize: 3 * root.size
            font.weight: Font.Light
            font.family: "Roboto"
        }
    }

    Item {
        id: textObject
        property alias font: textFontProxy.font
        property color color: Qt.rgba(0.15, 0.15, 0.15, 0.9)
        Text {
            id: textFontProxy
            font.pixelSize: 1.5 * root.size
            font.weight: Font.Light
            font.family: "Roboto"
        }
    }

    Item {
        id: controlObject
        property alias font: controlFontProxy.font
        property color color: Qt.rgba(0.15, 0.15, 0.15, 0.9)
        property real spacing: controlFontMetrics.height * 0.5
        FontMetrics {
            id: controlFontMetrics
            font: controlObject.font
        }
        Text {
            id: controlFontProxy
            font.family: "Roboto"
        }
    }

    Item {
        id: headingObject
        property alias font: headingFontProxy.font
        property color color: Qt.rgba(0.4, 0.4, 0.4, 1.0)
        property real size: 2.5 * font.size
        Text {
            id: headingFontProxy
            font.pixelSize: 2.5 * root.font.pixelSize
            font.weight: Font.Light
            font.family: "Roboto"
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
