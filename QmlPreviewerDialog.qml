import QtQuick 2.0
import QtQuick.Controls 2.0
import QtQuick.Layouts 1.1
import QtQuick.Dialogs 1.2
import Qt.labs.settings 1.0

Item {
    id: root

    signal start(url projectPath, url filePath)

    property url projectPath
    property url filePath
    property url source

    property string fileBaseName: {
        var split = filePath.toString().split("/")
        return split[split.length - 1]
    }

    width: 1600
    height: 900

    Component.onCompleted: {
        requestStart()
    }

    onFilePathChanged: {
        requestStart()
    }

    onProjectPathChanged: {
        requestStart()
    }

    function reload() {
        loader.source = ""
        var clippedName = filePath.toString().replace(projectPath.toString(), "")
        loader.source = "qrc:///qtqmlpreview/" + clippedName
    }

    function requestStart() {
        if(filePath.toString().length > 0 && projectPath.toString().length > 0) {
            start(projectPath, filePath)
        }
    }

    Settings {
        property alias qmlPreviewerProjectPath: root.projectPath
        property alias qmlPreviewerFilePath: root.filePath
    }

    Column {
        id: column
        anchors {
            left: parent.left
            top: parent.top
            bottom: parent.bottom
        }

        width: 300
        spacing: 16

        QmlPreviewerButton {
            text: "Select project path"
            subtext: projectPath.toString()

            onClicked: folderDialog.open()
        }

        QmlPreviewerButton {
            text: "Select file to preview"
            subtext: fileBaseName

            onClicked: fileDialog.open()
        }

        QmlPreviewerButton {
            text: "Refresh"
            onClicked: requestStart()
        }
    }

    Rectangle {
        anchors {
            top: parent.top
            bottom: parent.bottom
            left: column.right
            right: parent.right
        }
        Loader {
            id: loader
            anchors.fill: parent
        }
    }

    FileDialog {
        id: folderDialog

        selectFolder: true

        onAccepted: {
            projectPath = folder
        }
    }

    FileDialog {
        id: fileDialog

        onAccepted: {
            filePath = fileUrl
        }
    }
}
