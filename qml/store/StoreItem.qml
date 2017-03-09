import QtQuick 2.4
import QtQuick.Controls 2.0
import QtQuick.Controls.Material 2.0
import QtGraphicalEffects 1.0

Item {
    id: root

    signal clicked(string name)

    property string name
    property string description
    property string price
    property url imageUrl

    Material.theme: Material.Light

    clip: true

    width: 160
    height: 256

    Rectangle {
        id: background
        anchors.fill: parent
        color: Material.background
    }

    StoreShadow {
        source: background
        anchors.fill: background
        samples: 16
    }

    Rectangle {
        id: rectangle1
        anchors {
            bottom: titleText.top
            bottomMargin: 8
            top: parent.top
            right: parent.right
            left: parent.left
        }

        color: "#ffffff"

        Image {
            id: image1
            fillMode: Image.PreserveAspectCrop
            anchors.fill: parent
            source: root.imageUrl
        }
    }

    FontMetrics {
        id: defaultMetric
    }

    FontMetrics {
        id: titleMetrics
        font: titleText.font
    }

    Label {
        id: titleText
        clip: true
        anchors {
            bottom: descriptionText.top
            left: parent.left
            leftMargin: 8
        }

        font.pixelSize: 16

        text: root.name
    }

    LinearGradient {
        id: gradient
        property rect textRect: titleMetrics.boundingRect(titleText.text)
        anchors {
            top: titleText.top
            bottom: titleText.bottom
            right: parent.right
        }
        width: parent.width * 0.5
        start: Qt.point(0, 0)
        end: Qt.point(width, 0)
        visible: titleText.x + titleText.width > root.width - 8
        gradient: Gradient {
            GradientStop {
                color: "transparent"
                position: 0
            }
            GradientStop {
                color: background.color
                position: 1
            }
        }
    }

    Label {
        id: descriptionText
        anchors {
            left: parent.left
            right: parent.right
            bottom: priceText.top
            margins: 8
        }
        clip: true
        color: Material.shade(Material.background, Material.Shade100)

        text: root.description
    }

    Label {
        id: priceText
        anchors {
            right: parent.right
            rightMargin: 8
            bottom: parent.bottom
            bottomMargin: 8
        }

        color: (root.price.toLowerCase() === "free") ? Material.color(Material.Green) : Material.color(Material.Blue)

        text: root.price.toUpperCase()
    }

    MouseArea {
        id: mouseArea

        anchors.fill: parent

        onClicked: root.clicked(root.name)
    }
}

