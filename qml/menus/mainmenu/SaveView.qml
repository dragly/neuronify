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

    property string saveFolder: StandardPaths.writableLocation(StandardPaths.AppConfigLocation) + "/savedata"

    function refresh() {
        saveView.refreshing = true;
        refreshTimer.restart();
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
                        saveView.save(filePath)
                        saveView.requestScreenshot(function(result) {
                            result.saveToFile(imageFilename);
                            saveView.refresh();
                        });
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
                        saveView.save(filePath)
                        saveView.requestScreenshot(function(result) {
                            result.saveToFile(imageFilename);
                        });
                    }
                }
            }
        }
    }

    FolderListModel {
        id: folderModel
        nameFilters: ["*.png"]
        folder: "file://" + saveView.saveFolder
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
