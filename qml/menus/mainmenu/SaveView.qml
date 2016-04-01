import QtQuick 2.6
import QtQuick.Layouts 1.3
import "../../style"
import "../../io"
import ".."

import Neuronify 1.0

Item {
    id: saveView
    clip: true
    property bool isSave
    width: 200
    height: 100
    signal load(var filename)
    signal save(var filename)
    signal requestScreenshot(var callback)

    Heading {
        id: aboutHeading
        anchors {
            top: parent.top
            horizontalCenter: parent.horizontalCenter
            topMargin: Style.margin
        }
        text: isSave ? "Save" : "Load"
    }

    GridLayout{
        id: saveFileDialog
        property int padding: 10
        anchors{
            top: aboutHeading.bottom
            left: parent.left
            right: parent.right
            bottom: parent.bottom
            margins: padding
        }
        width : parent.width
        height: parent.height

        columns: 3
        columnSpacing: padding
        rowSpacing: padding
        Repeater{
            model : 6
            CustomFileIcon {
                name: index;
                saveFilename: "file://" + StandardPaths.writableLocation(StandardPaths.AppConfigLocation) + "/custom" + index + ".nfy";
                onClicked: {
                    if (isSave) {
                        save(saveFilename)

                        saveView.requestScreenshot(function(result) {
                            result.saveToFile("/tmp/something.png");
                        });
                        console.log("calling save from saveView")
                    } else {
                        load(saveFilename)
                        console.log("calling load from saveView")
                    }
                }
            }
        }


        //        CustomFileIcon{index: "2"; saveFilename: "file://" + StandardPaths.writableLocation(StandardPaths.AppConfigLocation) + "/custom2.nfy"}
        //        CustomFileIcon{index: "3"; saveFilename: "file://" + StandardPaths.writableLocation(StandardPaths.AppConfigLocation) + "/custom3.nfy"}
        //        CustomFileIcon{index: "4"; saveFilename: "file://" + StandardPaths.writableLocation(StandardPaths.AppConfigLocation) + "/custom4.nfy"}
        //        CustomFileIcon{index: "5"; saveFilename: "file://" + StandardPaths.writableLocation(StandardPaths.AppConfigLocation) + "/custom5.nfy"}
        //        CustomFileIcon{index: "6"; saveFilename: "file://" + StandardPaths.writableLocation(StandardPaths.AppConfigLocation) + "/custom6.nfy"}
    }
}
