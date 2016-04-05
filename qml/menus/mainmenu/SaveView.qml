import QtQuick 2.6
import QtQuick.Layouts 1.3
import Qt.labs.folderlistmodel 2.1

import "../../style"
import "../../io"
import ".."

import Neuronify 1.0

MainMenuPage {
    id: saveView
    clip: true
    property bool isSave
    property bool refreshing: false
    signal load(var filename)
    signal save(var filename)
    signal requestScreenshot(var callback)

    property url saveFolderUrl: StandardPaths.writableLocation(StandardPaths.AppConfigLocation, "savedata")
    property string saveFolder: Qt.resolvedUrl(saveFolderUrl)

    // TODO picture doesn't get properly updated when overwriting. Do we need to refresh the pixmap too?
    function refresh() {
        saveView.refreshing = true;
        refreshTimer.restart();
    }

    function saveStateImplementation(fileUrl, imageUrl) {
        saveView.save(fileUrl)
        saveView.requestScreenshot(function(result) {
            // needs to be local file because of type of "result"
            var imageFile = StandardPaths.toLocalFile(imageUrl);
            console.log("Saving image to " + imageFile);
            result.saveToFile(imageFile);
        });
    }

    title: isSave ? "Save" : "Load"

    width: 200
    height: 100

    GridLayout{
        id: saveFileDialog
        property int padding: 10
        anchors {
            fill: parent
            margins: Style.margin
        }
        width : parent.width
        height: parent.height

        columns: saveView.width > saveView.height ? 3 : 2
        columnSpacing: padding
        rowSpacing: padding


        Repeater {
            id: iconRepeater

            visible: !saveView.refreshing
            model: saveView.refreshing ? undefined : folderModel

            CustomFileIcon {
                basePath: saveView.saveFolder + "/" + fileBaseName
                onClicked: {
                    if (isSave) {
                        saveStateImplementation(filePath, imagePath);
                    } else {
                        saveView.load(filePath)
                    }
                }
            }
        }
        Repeater {
            visible: !saveView.refreshing
            model: 6 - folderModel.count
            CustomFileIcon {
                basePath: saveView.saveFolder + "/custom" + folderModel.count
                empty: true
                onClicked: {
                    if(saveView.isSave) {
                        saveStateImplementation(filePath, imagePath);
                    }
                }
            }
        }
    }

    FolderListModel {
        id: folderModel
        nameFilters: ["custom*.png"]
        folder: saveView.saveFolderUrl
    }

    Timer {
        id: refreshTimer
        interval: 100
        running: false
        repeat: false
        onTriggered: {
            saveView.refreshing = false;
        }
    }
}
