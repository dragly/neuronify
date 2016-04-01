import QtQuick 2.0
import QtQuick.Window 2.0
import QtQuick.Layouts 1.3
import "../.."
import "../style"

Rectangle {
        id: iconRoot
        signal clicked
        property var name
        property var saveFilename
        property var imageName
        Layout.fillWidth : true
        Layout.fillHeight: true
        width : 1
        height : 1
        //color : fileExists() ? "red" : "blue"
        color: Style.button.backgroundColor


        function fileExists(){
            var code = customFileManager.read(saveFilename);
            if(!code) {
                return false;
            } else {
                return true
            }
        }


        function imageExists(){
            var code = customFileManager.read("file://" + imageName);
            if(!code) {
                return false;
            } else {
                return true
            }
        }

        function refresh(){
            if (imageExists()){
                    iconImage.source = ("file://" + imageName)
                    saveText.text = ""
            }
        }

        Image{
            id: iconImage
            //source: "file://" + imageName
            source: imageExists() ? ("file://" + imageName) : ""
            width: parent.width
            height: parent.height
        }

        MouseArea{
            anchors.fill: parent
            enabled: (fileExists() || saveView.isSave)
            onClicked: {
                iconRoot.clicked()
                //iconImage.source = ("file://" + imageName)
                iconRoot.color = fileExists() ? Style.button.backgroundColor : "blue"
            }

        }
        Text {
            id: saveText
            text: fileExists() ? "" : "Empty file"
            font: Style.button.font
            renderType: Text.QtRendering
            color: Style.button.color
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.verticalCenter: parent.verticalCenter
        }

        FileManager {
            id: customFileManager

        }
    }

