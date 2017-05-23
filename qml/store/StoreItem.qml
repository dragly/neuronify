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
            bottom: textContainer.top
            bottomMargin: 8
            top: parent.top
            right: parent.right
            left: parent.left
        }

        color: "#ffffff"
        clip: true

        Image {
            id: image1
            fillMode: Image.PreserveAspectCrop
            anchors.centerIn: parent
            height: root.height
            width: height

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

    Item {
        id: textContainer
        anchors {
            bottom: parent.bottom
            left: parent.left
            right: parent.right
            margins: 16
        }

        height: titleMetrics.height * 2
        clip: true

        Column {
            anchors {
                top: parent.top
                left: parent.left
                right: parent.right
            }

            Label {
                id: titleText

                anchors {
                    left: parent.left
                    right: parent.right
                }

                clip: true

                font.pixelSize: 16
                wrapMode: Label.WrapAtWordBoundaryOrAnywhere

                text: root.name
            }

            Label {
                id: descriptionText

                anchors {
                    left: parent.left
                    right: parent.right
                }
                clip: true
                wrapMode: Label.WrapAtWordBoundaryOrAnywhere
                color: Material.shade(Material.foreground, Material.Shade100)

                text: root.description
            }

        }
    }

    //    LinearGradient {
    //        id: gradient
    //        property rect textRect: titleMetrics.boundingRect(titleText.text)
    //        anchors {
    //            top: titleText.top
    //            bottom: titleText.bottom
    //            right: parent.right
    //        }
    //        width: parent.width * 0.5
    //        start: Qt.point(0, 0)
    //        end: Qt.point(width, 0)
    //        visible: titleText.x + titleText.width > root.width - 8
    //        gradient: Gradient {
    //            GradientStop {
    //                color: "transparent"
    //                position: 0
    //            }
    //            GradientStop {
    //                color: background.color
    //                position: 1
    //            }
    //        }
    //    }



    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        onClicked: root.clicked(root.name)
        cursorShape: Qt.PointingHandCursor
    }

    states: [
        State {
            when: mouseArea.containsMouse
            name: "hovered"
            PropertyChanges {
                target: textContainer
                height: root.height * 0.7
            }
        }
    ]
    transitions: [
        Transition {
            NumberAnimation {
                id: heightAnimation
                properties: "height"
                duration: 300
                easing.type: Easing.OutQuad
            }
        }
    ]
}

