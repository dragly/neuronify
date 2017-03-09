import QtQuick 2.0
import QtGraphicalEffects 1.0
import QtQuick.Controls 2.1

Item {
    FontLoader {
        name: "material"
        source: "https://github.com/google/material-design-icons/raw/master/iconfont/MaterialIcons-Regular.ttf"
    }

//    Text {
//        anchors.centerIn: parent
//        text: 'back'
//        font.family: "Material Icons"
//        color: "white"
//        font.pixelSize: 48
//    }

    Slider {
        id: slider
        from: 24
        to: 1024
        value: 128
    }

    Image {
        id: image

        property int renderPower: Math.ceil(Math.log(width) / Math.LN2)
        property int renderSize: Math.pow(2, renderPower)

        smooth: true
        antialiasing: true

        onRenderSizeChanged: console.log("Logs", width, renderPower, renderSize)

        anchors.centerIn: parent
        width: slider.value
        height: width
        visible: false
        sourceSize: Qt.size(renderSize, renderSize)
        source: "qrc:/images/material-icons/navigation/svg/production/ic_arrow_back_24px.svg"
    }

    ColorOverlay {
        anchors.fill: image
        source: image
        color: "red"
        smooth: true
        antialiasing: true
    }
}
