import QtQuick 2.0
import "../../style"
import "../"


Item {
    id: aboutView

    width: 200
    height: 100

    Heading {
        id: aboutHeading
        anchors {
            top: parent.top
            horizontalCenter: parent.horizontalCenter
            topMargin: Style.baseMargin
        }
        text: "About"
    }

    Flickable {
        id: aboutFlickable
        anchors {
            left: parent.left
            right: parent.right
            top: aboutHeading.bottom
            bottom: parent.bottom
            margins: Style.baseMargin
        }

        Text {
            id: aboutText

            width: aboutFlickable.width

            font.pixelSize: Style.font.size
            wrapMode: Text.WrapAtWordBoundaryOrAnywhere
            color: Style.font.color
            textFormat: Text.RichText
            text: "<p>"+
                  "Neuronify is a neural network simulator developed by PhD students in CINPLA." +
                  "</p>" +
                  "<p>" +
                  "The model used in Neuronify is a integrate-and-fire model, which gives a simple description of neurons in large networks. " +
                  "These networks can be used to explain properties that arise out of the network.  " +
                  "</p>"
        }

        Image {
            anchors {
                bottom: parent.bottom
                horizontalCenter: parent.horizontalCenter
                topMargin: Style.baseMargin
            }

            width: parent.width*0.4; height: parent.height*0.5
            fillMode: Image.PreserveAspectFit
            smooth: true
            source: "../../images/logo.png"
        }



    }
}



