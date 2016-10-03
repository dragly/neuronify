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
    property url imageUrl: "qrc:/images/store/ic_search_black_48dp.png"

    width: 320
    height: 480

    Rectangle {
        id: background
        anchors.fill: parent
        color: !mouseArea.pressed ? "#fff" : "#ddd"
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

    Label {
        id: titleText
        clip: true
        anchors {
            bottom: descriptionText.top
            right: parent.right
            rightMargin: 8
            left: parent.left
            leftMargin: 8
        }

        font.pixelSize: defaultMetric.font.pixelSize * 1.6

        text: root.name
    }

    LinearGradient {
        anchors {
            top: titleText.top
            bottom: titleText.bottom
            right: titleText.right
        }
        width: titleText.width * 0.5
        start: Qt.point(0, 0)
        end: Qt.point(width, 0)
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
        color: Material.shade(Material.foreground, Material.Shade100)

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

