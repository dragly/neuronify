import QtQuick 2.0
import QtGraphicalEffects 1.0
import QtQuick.Controls 2.1
import QtQuick.Controls.Material 2.1

Item {
    id: root
    property string name
    property string category: "navigation"
    property color color: Material.foreground
    opacity: color.a

    Image {
        id: image

        property int renderPower: Math.ceil(Math.log(width) / Math.LN2)
        property int renderSize: Math.pow(2, renderPower)

        anchors.fill: parent
        smooth: true
        antialiasing: true
        visible: false
        fillMode: Image.PreserveAspectFit
        sourceSize: Qt.size(renderSize, renderSize)
        source: "qrc:/images/material-icons/" + category + "/svg/production/ic_" + name.replace(/ /g, "_") + "_24px.svg"
    }

    ColorOverlay {
        anchors.fill: parent
        source: image
        color: Qt.rgba(root.color.r, root.color.g, root.color.b) // NOTE: Dropping alpha
        smooth: true
        antialiasing: true
    }
}

