import QtQuick 2.0
import QtQuick.Controls 1.4
import QtQuick.Dialogs 1.2
import QtQuick.Layouts 1.1
import Qt.labs.settings 1.0
import QtQuick.XmlListModel 2.0
import Qt.labs.folderlistmodel 2.1

Rectangle {
    id: root

    signal changeQrcPaths(var qrcPaths)

    readonly property string rootPath: "file::/qtqmlpreview"
    property var qrcPaths: []
    property string qrcPathsStringified
    property url filePath
    property url source
    property bool showHud: true
    property bool showBorder: true
    property bool enableDragging: true
    property bool enableResizing: true
    property alias backgroundColor: colorDialog.color

    function wrap(value) {
        return (value + 0.5) % 1.0
    }

    function baseName(file) {
        var split = file.toString().split("/")
        return split[split.length - 1]
    }

    function cleanPath(path) {
        return path.toString().replace(/^file:\/\//, "")
    }

    width: 1600
    height: 900

    onQrcPathsStringifiedChanged: {
        var paths = JSON.parse(qrcPathsStringified)
        var newPaths = []
        for(var i in paths) {
            newPaths.push(Qt.resolvedUrl(paths[i]))
        }
        qrcPaths = newPaths
        console.log("Parsed as", qrcPaths)
    }

    onQrcPathsChanged: {
        var stringPaths = []
        for(var i in qrcPaths) {
            stringPaths.push(qrcPaths[i].toString())
        }
        qrcPathsStringified = JSON.stringify(stringPaths)
        console.log("Stringified as", qrcPathsStringified)
        notifyChangeQrcPaths()
    }

    onFilePathChanged: {
        reload()
    }

    function refresh() {
        notifyChangeQrcPaths()
        //        var currentPath = folderListModel.folder
    }

    function refreshFileView() {
        console.log("Refresh file view")
        folderListModel.folder = ""
        folderListModel.folder = rootPath
    }

    function reload() {
        console.log("Reload")
        loader.active = false
        loader.source = filePath.toString().replace("file:", "qrc")
        loader.active = true
//        loader.source = ""
    }

    function notifyChangeQrcPaths() {
        console.log("Change qrc paths")
        if(additionalCheckBox.checked) {
            changeQrcPaths(qrcPaths)
        } else {
            changeQrcPaths([])
        }
    }

    Settings {
        property alias filePath: root.filePath
        property alias qrcPaths: root.qrcPathsStringified
        property alias backgroundColor: colorDialog.color
        property alias canvasX: canvas.x
        property alias canvasY: canvas.y
        property alias canvasWidth: canvas.width
        property alias canvasHeight: canvas.height
        property alias showBorder: root.showBorder
        property alias enableResizing: root.enableResizing
        property alias enableDragging: root.enableDragging
        property alias width: root.width
        property alias height: root.height

        category: "qmlPreviewer"
    }

    Rectangle {
        id: pane
        anchors {
            left: parent.left
            top: parent.top
            bottom: parent.bottom
            leftMargin: showHud ? 0 : -width
        }

        width: 300
        color: "#cdcdcd"

        Flickable {
            anchors.fill: parent
            contentHeight: column.height

            Column {
                id: column
                spacing: 16

                anchors {
                    top: parent.top
                    left: parent.left
                    right: parent.right
                    margins: 24
                }

                Column {
                    anchors {
                        left: parent.left
                        right: parent.right
                    }

                    spacing: 8

                    //                    visible: folderListModel.folder != ""

                    Label {
                        text: "Files in QRC:"
                    }

                    Row {
                        spacing: 8
                        Button {
                            id: upButton
                            text: "Up"
                            enabled: folderListModel.folder != rootPath
                            onClicked: {
                                if(!folderListModel.parentFolder) {
                                    return
                                }

                                console.log(folderListModel.folder, folderListModel.parentFolder)
                                folderListModel.folder = folderListModel.parentFolder
                            }
                        }

                        Label {
                            anchors.verticalCenter: upButton.verticalCenter
                            text: folderListModel.folder.toString().replace("^file::/qtqmlpreview", "")
                        }

                    }

                    Rectangle {
                        anchors {
                            left: parent.left
                            right: parent.right
                        }
                        height: 300
                        color: "#efefef"
                        ListView {
                            id: fileView
                            anchors.fill: parent
                            clip: true

                            model: FolderListModel {
                                id: folderListModel
                                showDirsFirst: true
                                showHidden: true
                                folder: ""
                                nameFilters: "*.qml"
                                rootFolder: "qrc:/"

                                onFolderChanged: {
                                    console.log("Current folder:", folder)
                                }
                            }
                            delegate: Rectangle {
                                anchors {
                                    left: parent.left
                                    right: parent.right
                                }

                                color: ListView.isCurrentItem || fileMouseArea.containsMouse ? "#bcbcbc" : "transparent"
                                height: fileRow.height + 2 * fileRow.anchors.margins

                                Row {
                                    id: fileRow
                                    anchors {
                                        left: parent.left
                                        right: parent.right
                                        verticalCenter: parent.verticalCenter
                                        margins: 8
                                    }
                                    spacing: 8
                                    Image {
                                        source: fileIsDir ? "images/ic_folder_black_24px.svg" : "images/ic_insert_drive_file_black_24px.svg"
                                        width: 16
                                        height: 16
                                        smooth: true
                                        antialiasing: true
                                    }
                                    Label {
                                        id: fileLabel
                                        anchors {
                                            verticalCenter: parent.verticalCenter
                                        }
                                        wrapMode: Label.WrapAtWordBoundaryOrAnywhere
                                        text: fileName
                                    }
                                }
                                MouseArea {
                                    id: fileMouseArea
                                    anchors.fill: parent
                                    onClicked: {
                                        if(fileIsDir) {
                                            folderListModel.folder = fileURL
                                        } else {
                                            fileView.currentIndex = index
                                            console.log("File URL", fileURL, filePath)
                                            root.filePath = filePath.replace(":/", "qrc:/")
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                Row {

                    spacing: 8

                    Label {
                        anchors.verticalCenter: parent.verticalCenter
                        text: "Background color:"
                    }

                    Rectangle {
                        width: 64
                        height: 32
                        color: colorDialog.color
                        MouseArea {
                            anchors.fill: parent
                            onClicked: colorDialog.open()
                        }
                    }
                }

                CheckBox {
                    id: additionalCheckBox
                    text: "Include additional QRC files"
                }

                ListView {
                    anchors {
                        left: parent.left
                        right: parent.right
                    }

                    visible: additionalCheckBox.checked

                    height: 160

                    model: root.qrcPaths
                    delegate: Rectangle {
                        anchors {
                            left: parent.left
                            right: parent.right
                        }

                        color: ListView.isCurrentItem ? "#bcbcbc" : "#efefef"
                        height: fileLabel.height + 2 * fileLabel.anchors.margins

                        Label {
                            id: fileLabel
                            anchors {
                                left: parent.left
                                right: parent.right
                                verticalCenter: parent.verticalCenter
                                margins: 8
                            }
                            wrapMode: Label.WrapAtWordBoundaryOrAnywhere
                            text: baseName(modelData)
                        }
                        MouseArea {
                            anchors.fill: parent
                            onClicked: {
                                var paths = root.qrcPaths
                                paths.splice(index, 1)
                                root.qrcPaths = paths
                            }
                        }
                    }
                }

                Button {
                    visible: additionalCheckBox.checked

                    text: "Add .qrc"
                    onClicked: qrcDialog.open()
                }
            }
        }

        Behavior on anchors.leftMargin {
            NumberAnimation {
                duration: 800
                easing.type: Easing.InOutQuad
            }
        }

    }

    Rectangle {
        anchors {
            top: parent.top
            bottom: parent.bottom
            left: pane.right
            right: parent.right
        }
        color: colorDialog.color
        Item {
            id: canvas
            width: 640
            height: 480

            MouseArea {
                anchors.fill: parent
                drag.target: parent
                enabled: enableDragging
            }

            ResizeRectangle {
                target: parent
                enabled: enableResizing
            }

            Item {
                anchors {
                    fill: parent
                    margins: 24
                }

                Loader {
                    id: loader
                    anchors.fill: parent
                    clip: true
                }

                Rectangle {
                    anchors.fill: loader
                    color: "transparent"
                    border {
                        color: Qt.hsla(wrap(backgroundColor.hslHue), 0.1, wrap(backgroundColor.hslLightness))
                        width: root.showBorder ? 1.0 : 0.0
                    }
                }
            }

            Rectangle {
                anchors.fill: canvas
                color: "transparent"
                border {
                    color: Qt.hsla(wrap(backgroundColor.hslHue), 0.1, wrap(backgroundColor.hslLightness))
                    width: root.showBorder ? 2.0 : 0.0
                }
            }
        }

        Column {
            id: hideColumn
            anchors {
                right: parent.right
                top: parent.top
                margins: 24
            }
            spacing: 8

            opacity: (showHud || hideMouseArea.containsMouse) ? 1.0 : 0.0

            CheckBox {
                id: hudCheckBox
                text: "Show HUD"
                checked: root.showHud

                onCheckedChanged: {
                    notifyChangeQrcPaths()
                }

                Binding {
                    target: hudCheckBox
                    property: "checked"
                    value: root.showHud
                }

                Binding {
                    target: root
                    property: "showHud"
                    value: hudCheckBox.checked
                }
            }

            CheckBox {
                id: borderCheckBox
                text: "Show border"
                checked: root.showBorder

                Binding {
                    target: borderCheckBox
                    property: "checked"
                    value: root.showBorder
                }

                Binding {
                    target: root
                    property: "showBorder"
                    value: borderCheckBox.checked
                }
            }

            CheckBox {
                id: dragCheckBox
                text: "Enable dragging"
                checked: root.enableDragging

                Binding {
                    target: dragCheckBox
                    property: "checked"
                    value: root.enableDragging
                }

                Binding {
                    target: root
                    property: "enableDragging"
                    value: dragCheckBox.checked
                }
            }

            CheckBox {
                id: resizeCheckBox
                text: "Enable resizing"
                checked: root.enableResizing

                Binding {
                    target: resizeCheckBox
                    property: "checked"
                    value: root.enableResizing
                }

                Binding {
                    target: root
                    property: "enableResizing"
                    value: resizeCheckBox.checked
                }
            }

            Behavior on opacity {
                NumberAnimation {
                    duration: 800
                    easing.type: Easing.InOutQuad
                }
            }
        }

        MouseArea {
            id: hideMouseArea
            anchors.fill: hideColumn
            acceptedButtons: Qt.NoButton
            hoverEnabled: true
        }
    }

    ColorDialog {
        id: colorDialog
        color: "white"
    }

    FileDialog {
        id: folderDialog

        selectFolder: true

        onAccepted: {
            projectPath = folder
        }
    }

    FileDialog {
        id: qrcDialog

        nameFilters: "*.qrc"

        onAccepted: {
            var paths = root.qrcPaths
            paths.push(fileUrl)
            console.log("Set new paths", paths)
            root.qrcPaths = paths
        }
    }

    FileDialog {
        id: fileDialog

        onAccepted: {
            filePath = fileUrl
        }
    }
}
