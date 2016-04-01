import QtQuick 2.0
import QtQuick.Window 2.0
import QtQuick.Layouts 1.3
import "../.."


Rectangle {
        id: iconRoot
        signal clicked
        property var name
        property var saveFilename
        Layout.fillWidth : true
        Layout.fillHeight: true
        width : 1
        height : 1
        color : fileExists() ? "red" : "blue"


        function fileExists(){
            var code = customFileManager.read(saveFilename);
            if(!code) {
                return false;
            } else {
                return true
            }
        }

        MouseArea{
            anchors.fill: parent
//            enabled: fileExists()
            onClicked: {
                iconRoot.clicked()
            }

        }
        Text {
            id: saveText
            text: parent.name
            color: "black"
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.verticalCenter: parent.verticalCenter
        }

        FileManager {
            id: customFileManager

        }
    }

