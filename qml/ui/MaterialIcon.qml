import QtQuick 2.0
import QtGraphicalEffects 1.0
import QtQuick.Controls 2.1

Item {
    id: root
    property string name
    property color color

    Image {
        id: image

        property int renderPower: Math.ceil(Math.log(width) / Math.LN2)
        property int renderSize: Math.pow(2, renderPower)

        anchors.fill: parent
        smooth: true
        antialiasing: true
        visible: false
        sourceSize: Qt.size(renderSize, renderSize)
        source: "qrc:/images/material-icons/navigation/svg/production/ic_" + name + "_24px.svg"
    }

    ColorOverlay {
        anchors.fill: parent
        source: image
        color: root.color
        smooth: true
        antialiasing: true
    }
}

