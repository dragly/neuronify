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
    property real margin: 2 * size
    property real baseMargin: margin
    property real scale: 1.0
    property real spacing: size * 0.5

    property alias text: textObject
    property alias heading: headingObject
    property alias font: textFontProxy.font
    property alias button: menuButtonObject
    property alias control: controlObject
    property alias menu: menuObject

    property alias meter: metersObject

    property alias color: colorsObject
    property alias border: borderObject

    property real playbackSpeed
    property real workspaceScale

    QtObject {
        id: colorsObject
        property color background: "#f7fbff"
        property color foreground: "#deebf7"
        property alias border: borderObject.color
    }

    QtObject {
        id: metersObject
        property color background: "#deebf7"
        property color edge: border.color
        property alias border: meterBorderObject
    }

    QtObject {
        id: meterBorderObject
        property color color: "#9ecae1"
        property real width: 2.0
    }


    QtObject {
        id: borderObject
        property color color: "#9ecae1"
        property color lightColor: "#d5e8f2"
        property real width: 2.0
    }

    FontMetrics {
        id: defaultMetrics
    }

    Item {
        id: textObject
        property alias font: textFontProxy.font
        property color color: Qt.rgba(0.15, 0.15, 0.15, 0.9)
        Text {
            id: textFontProxy
            font.pixelSize: 2.0 * root.size
            font.weight: Font.Light
            font.family: "Roboto Light, Roboto"
        }
    }

    Item {
        id: menuObject

        property alias button: menuButtonObject
        property alias text: menuTextObject

        Item {
            id: menuButtonObject
            property alias font: menuButtonFontProxy.font
            property color color: Qt.rgba(0.15, 0.15, 0.15, 1.0)
            property color backgroundColor: "#dedede"
            Text {
                id: menuButtonFontProxy
                font.pixelSize: 3 * root.size
                font.weight: Font.Light
                font.family: "Roboto Light, Roboto"
            }
        }

        Item {
            id: menuTextObject
            property alias font: menuTextFontProxy.font
            property color color: Qt.rgba(0.15, 0.15, 0.15, 1.0)
            property color backgroundColor: "#dedede"
            Text {
                id: menuTextFontProxy
                font.pixelSize: 2.5 * root.size
                font.family: "Roboto Light, Roboto"
            }
        }
    }

    Item {
        id: controlObject

        property alias text: controlTextProxy
        property alias subText: controlSubTextProxy
        property alias heading: controlHeadingProxy
        property alias font: controlTextProxy.font
        property alias fontMetrics: controlFontMetrics

        property color color: Qt.rgba(0.15, 0.15, 0.15, 0.9)
        property real spacing: controlFontMetrics.height * 0.5

        FontMetrics {
            id: controlFontMetrics
            font: controlObject.font
        }

        Text {
            id: controlTextProxy
            color: "#222"
            font.family: "Roboto"
            font.pixelSize: Math.min(root.windowHeight * 0.042, defaultMetrics.font.pixelSize)
        }
        Text {
            id: controlSubTextProxy
            color: "#999"
            font.family: "Roboto"
            font.pixelSize: controlObject.font.pixelSize * 0.8
        }
        Text {
            id: controlHeadingProxy
            color: "#000"
            font.family: "Roboto Light, Roboto"
            font.weight: Font.Light
            font.pixelSize: controlObject.font.pixelSize * 1.2
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
            font.family: "Roboto Light, Roboto"
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
            device = "dekstop";
            var base = Math.min(root.windowWidth, root.windowHeight);
            size = base * 0.02;
        }
    }
}
