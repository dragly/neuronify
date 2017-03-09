import QtQuick 2.5
import QtQuick.Controls 2.1
import QtQuick.Controls.Material 2.1
import QtQuick.Dialogs 1.0
import QtQuick.Layouts 1.3
import QtQuick.Particles 2.0
import QtQuick.Window 2.1

import QtCharts 2.1
import QtMultimedia 5.5
import Qt.labs.settings 1.0
import Qt.labs.folderlistmodel 2.1
import Qt.labs.platform 1.0

import Neuronify 1.0
import CuteVersioning 1.0
import QtGraphicalEffects 1.0

import "qrc:/qml/backend"
import "qrc:/qml/controls"
import "qrc:/qml/hud"
import "qrc:/qml/io"
import "qrc:/qml/menus/filemenu"
import "qrc:/qml/menus/mainmenu"
import "qrc:/qml/tools"
import "qrc:/qml/store"
import "qrc:/qml/style"
import "qrc:/qml/ui"

Item {
    id: root

    property bool dragging: false
    property url latestFolder: StandardPaths.writableLocation(StandardPaths.DocumentsLocation) + "/neuronify"

    state: "welcome"

    Settings {
        property alias latestFolder: root.latestFolder
    }

    DownloadManager {
        id: _downloadManager
    }

    Parse {
        id: parse
        debug: true
        serverUrl: "https://parseapi.back4app.com/"
        applicationId: "JffGes20AXUtdN9B6E1RkkHaS7DOxVmxJFSJgLoN"
        restApiKey: "bBKStu7bqeyWFTYFfM5OIes255k9XEz2Voe4fUxS"
    }

    Settings {
        id: settings
        category: "parse"
        property alias sessionToken: parse.sessionToken
    }

    Neuronify {
        id: neuronify
        anchors {
            left: parent.left
            right: parent.right
            top: parent.top
            bottom: parent.bottom
        }
        clip: true
        autoPause: root.state != "view" && root.state != "create"
    }

    Rectangle {
        id: leftMenu

        property real textOpacity: 1.0

        anchors {
            left: parent.left
            top: parent.top
            bottom: parent.bottom
            leftMargin: 0
        }

        width: 128

        Material.theme: Material.Dark

//        color: "#1782C2"
//        color: Material.color(Material.Cyan)
        color: Material.primary
        z: 40

        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
        }

        Image {
            id: logo
            anchors {
                left: parent.left
                right: parent.right
                top: parent.top
                margins: parent.width * 0.2
                topMargin: 48
            }
            fillMode: Image.PreserveAspectFit
            height: width
            source: "qrc:/images/logo/logo-no-background.png"
            mipmap: true
        }

        Text {
            id: logoText
            color: "white"
            font.pixelSize: 24
            font.family: Style.font.family
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WrapAtWordBoundaryOrAnywhere
            text: "Neuronify\n" + Version.latestTag
        }

        ShaderEffectSource {
            id: logoTextCopy
            anchors {
                left: parent.left
                right: parent.right
                top: logo.bottom
                //                top: parent.top
                margins: 8
            }
            height: width * logoText.height / logoText.width
            hideSource: true
            sourceItem: logoText
            smooth: true
            antialiasing: true
        }

        Column {
            id: menuColumn
            anchors {
                left: parent.left
                right: parent.right
                top: logoTextCopy.bottom
                topMargin: 48
            }
            spacing: 24
            Repeater {
                model: ListModel {
                    ListElement {
                        state: "welcome"
                        name: "Simulations"
                    }
                    ListElement {
                        state: "view"
                        name: "View"
                    }
                    ListElement {
                        state: "creation"
                        name: "Create"
                    }
                    //                    ListElement {
                    //                        state: "save"
                    //                        name: "Save"
                    //                    }
                    ListElement {
                        state: "help"
                        name: "Help"
                    }
                }
                MouseArea {
                    anchors {
                        left: parent.left
                        right: parent.right
                    }
                    height: menuItemColumn.height
                    onClicked: {
                        root.state = model.state
                    }
                    Column {
                        id: menuItemColumn
                        anchors {
                            left: parent.left
                            right: parent.right
                        }

                        spacing: 8
                        Rectangle {
                            anchors.horizontalCenter: parent.horizontalCenter
                            width: parent.width / 2
                            height: width
                            radius: width / 4
                            color: root.state == model.state ? "white" : "transparent"
                            border.width: parent.width * 0.04
                            border.color: "white"
                        }
                        Text {
                            anchors {
                                left: parent.left
                                right: parent.right
                            }
                            horizontalAlignment: Text.AlignHCenter
                            color: "white"
                            opacity: leftMenu.textOpacity
                            font.pixelSize: 12
                            text: model.name
                        }
                    }
                }
            }
        }

        states: [
            State {
                name: "small"
                PropertyChanges { target: leftMenu; width: 72 }
                PropertyChanges { target: logoTextCopy; opacity: 0.0 }
                PropertyChanges { target: leftMenu; textOpacity: 0.0 }
            },
            State {
                name: "hidden"
                AnchorChanges {
                    target: leftMenu
                    anchors {
                        left: undefined
                        right: parent.left
                    }
                }
            }

        ]

        transitions: [
            Transition {
                NumberAnimation {
                    target: leftMenu
                    properties: "width,textOpacity"
                    duration: 400
                    easing.type: Easing.InOutQuad
                }
                NumberAnimation {
                    target: leftMenuShadow
                    properties: "opacity"
                    duration: 0
                }
                AnchorAnimation {
                    duration: 0
                }
            },
            Transition {
                from: "hidden"
                to: ""
                AnchorAnimation {
                    duration: 400
                    easing.type: Easing.InOutQuad
                }
            }

        ]
    }

    Item {
        id: fileView
        anchors {
            left: leftMenu.right
            top: parent.top
            bottom: parent.bottom
            right: parent.right
        }
        z: 39

        Material.theme: Material.Dark

        Item {
            id: fileViewContent
            anchors.fill: parent

            Rectangle {
                id: background
                anchors.fill: parent
                color: leftMenu.color
                opacity: 1.0
            }

//            Blend {
//                anchors.fill: parent
//                source: blur
//                foregroundSource: background
//                mode: "multiply"
//            }

            ShaderEffectSource {
                id: neuronifySource
                anchors.fill: parent
                visible: false
                sourceItem: neuronify.shaderEffectItem
            }

            GaussianBlur {
                id: blur
                anchors.fill: parent
                radius: 48
                samples: 64
                source: neuronifySource
                opacity: 0.2
            }

            MaterialIcon {
                id: backButton
                anchors {
                    left: parent.left
                    top: parent.top
                    leftMargin: 16
                    topMargin: 24
                }

                width: 48
                height: 48
                color: "white"
                name: "arrow_back"

                MouseArea {
                    anchors.fill: parent
                    onClicked: {
                        root.state = "view"
                    }
                }
            }

            Item {
                id: fileViewMenu
                anchors {
                    left: parent.left
                    leftMargin: 8
                    top: backButton.bottom
                    topMargin: 24
                }
                width: 196
                height: viewColumn.height

                Rectangle {
                    anchors {
                        right: parent.right
                        top: parent.top
                        bottom: parent.bottom
                    }

                    width: 2
                    color: "white"
                    opacity: 0.2
                }

                FileMenu {
                    id: viewColumn
                    property string currentName
                    currentIndex: 2

                    anchors {
                        left: parent.left
                        right: parent.right
                    }

                    Component.onCompleted: {
                        reloadComponent()
                    }

                    function reloadComponent() {
                        currentName = children[currentIndex].name
                        stackView.replace(children[currentIndex].component)
                    }

                    onCurrentIndexChanged: {
                        reloadComponent()
                    }

                    FileMenuItem {
                        name: "New"
                        component: Item {
                            Column {
                                anchors {
                                    left: parent.left
                                    right: parent.right
                                }

                                Flow {
                                    anchors {
                                        left: parent.left
                                        right: parent.right
                                    }

                                    spacing: 16
                                    StoreItem {
                                        name: "Blank simulation"
                                        description: "Start with a blank canvas."
                                        onClicked: {
                                            neuronify.loadSimulation("qrc:/simulations/empty/empty.nfy")
                                            root.state = "view"
                                        }
                                    }

                                    Repeater {
                                        model: [
                                            {folder: "qrc:/simulations/tutorial/tutorial_1_intro"},
                                            {folder: "qrc:/simulations/tutorial/tutorial_2_circuits"},
                                            {folder: "qrc:/simulations/tutorial/tutorial_3_creation"},
                                            {folder: "qrc:/simulations/items/neurons/leaky"},
                                            {folder: "qrc:/simulations/items/neurons/inhibitory"},
                                            {folder: "qrc:/simulations/items/neurons/adaptation"},
                                            {folder: "qrc:/simulations/items/visualInput"},
                                            {folder: "qrc:/simulations/items/generators"},
                                            {folder: "qrc:/simulations/items/frPlot"},
                                            {folder: "qrc:/simulations/mix/lateral_inhibition"},
                                            {folder: "qrc:/simulations/mix/recurrent_inhibition"},
                                            {folder: "qrc:/simulations/mix/reciprocal_inhibition"},
                                            {folder: "qrc:/simulations/mix/disinhibition"},
                                            {folder: "qrc:/simulations/mix/rythm_transformation"},
                                            {folder: "qrc:/simulations/mix/prolonged_activity"},
                                            {folder: "qrc:/simulations/mix/lateral_inhibition_1"},
                                            {folder: "qrc:/simulations/mix/lateral_inhibition_2"},
                                            {folder: "qrc:/simulations/mix/input_summation"},
                                            {folder: "qrc:/simulations/sterratt/if_response"},
                                            {folder: "qrc:/simulations/sterratt/refractory_period"},
                                        ]
                                        StoreItem {
                                            SimulationLoader {
                                                id: loader
                                                folder: modelData.folder
                                            }

                                            name: loader.item.name
                                            description: loader.item.description
                                            imageUrl: loader.item.screenshotSource
                                        }
                                    }
                                }
                            }
                        }
                    }

                    FileMenuItem {
                        name: "Open"
                        component: Item {
                            Column {
                                anchors  {
                                    left: parent.left
                                    right: parent.right
                                }

                                spacing: 16
                                Button {
                                    Material.theme: Material.Light
                                    text: "From computer"
                                    onClicked: {
                                        openFileDialog.open()
                                    }
                                }

                                FolderDialog {
                                    id: openFileDialog
                                    currentFolder: root.latestFolder
                                    onAccepted: {
                                        neuronify.loadSimulation(folder + "/simulation.nfy")
                                        root.state = "view"
                                    }
                                }

                                FileMenuHeading {
                                    text: "Recent"
                                }

                                Flow {
                                    anchors {
                                        left: parent.left
                                        right: parent.right
                                    }
                                }
                            }
                        }
                    }

                    FileMenuItem {
                        name: "Save"
                        component: Item {
                            id: saveRoot
                            Column {
                                spacing: 16
                                width: Math.min(480, parent.width)

                                Label {
                                    text: qsTr("Name:")
                                }

                                TextField {
                                    id: saveName
                                    anchors {
                                        left: parent.left
                                        right: parent.right
                                    }
                                    validator: RegExpValidator {
                                        regExp: /[a-zA-Z0-9\-\_]+/
                                    }

                                    placeholderText: "e.g. MySimulation"
                                }

                                Label {
                                    text: "Screenshot preview:"
                                }

                                ShaderEffectSource {
                                    id: effectSource
                                    readonly property real aspectRatio: neuronify.width / neuronify.height
                                    width: parent.width * 0.6
                                    height: width / aspectRatio
//                                    sourceRect: Qt.rect(neuronify.width / 2 - width / 2,
//                                                        neuronify.height / 2 - height / 2,
//                                                        width,
//                                                        height)
                                    sourceItem: neuronify
                                }
                                Label {
                                    text: "Location:"
                                }

                                RowLayout {
                                    anchors {
                                        left: parent.left
                                        right: parent.right
                                    }
                                    spacing: 16

                                TextField {
                                    Layout.fillWidth: true
                                    readOnly: true
                                    text: latestFolder.toString().replace("file://", "")
                                }

                                Button {
                                    Material.theme: Material.Light
                                    width: 120
                                    text: "Change"
                                    onClicked: {
                                        saveFolderDialog.open()
                                    }
                                }

                                }

                                Button {
                                    anchors {
                                        right: parent.right
                                    }

                                    Material.theme: Material.Light
                                    width: 120
                                    enabled: saveName.acceptableInput
                                    text: qsTr("Save")

                                    onClicked: {
                                        neuronify.saveState(latestFolder + "/" + saveName.text + "/simulation.nfy")
                                        neuronify.saveScreenshot(latestFolder + "/" + saveName.text + "/screenshot.png")
                                        root.state = "view"
                                    }
                                }

                                FolderDialog {
                                    id: saveFolderDialog
                                    currentFolder: root.latestFolder
                                    onAccepted: {
                                        root.latestFolder = folder
                                    }
                                }
                            }
                        }

                    }
                    FileMenuItem {
                        name: "Download"
                        component: Flickable {
                            contentHeight: column.height + 64
                            clip: true

                            flickableDirection: Flickable.VerticalFlick
                            ScrollBar.vertical: ScrollBar {}

                            Component.onCompleted: {
                                communityProgressBar.visible = true
                                parse.get("Simulation", function(response) {
                                    communityProgressBar.visible = false
                                    communityRepeater.model = response.results
                                })
                            }

                            Component {
                                id: simulationComponent
                                StoreSimulation {
                                    downloadManager: _downloadManager
                                    onRunClicked: {
                                        neuronify.loadSimulation(fileUrl)
                                        root.state = "view"
                                        stackView.pop()
                                    }
                                }
                            }

                            Column {
                                id: column
                                anchors {
                                    left: parent.left
                                    right: parent.right
                                }

                                spacing: 16

                                FolderListModel {
                                    id: communityFolderModel
                                    folder: StandardPaths.writableLocation(StandardPaths.AppDataLocation) + "/community"
                                    showFiles: false
                                    showDirs: true
                                    showOnlyReadable: true
                                }

                                FileMenuHeading {
                                    text: "Installed"
                                    visible: communityFolderModel.count > 0
                                }

                                Flow {
                                    anchors {
                                        left: parent.left
                                        right: parent.right
                                    }

                                    spacing: 16
                                    visible: communityFolderModel.count > 0

                                    Repeater {
                                        model: communityFolderModel
                                        delegate: StoreItem {

                                            property var objectData: JSON.parse(FileIO.readSynchronously(model.fileURL + "/info.json"))

                                            width: 160
                                            height: 256
                                            name: objectData.name
                                            description: objectData.description
                                            imageUrl: model.fileURL + "/screenshot.png"
                                            onClicked: {
                                                console.log("Pushing", JSON.stringify(objectData))
                                                stackView.push(simulationComponent)
                                                stackView.currentItem.objectData = objectData
                                            }
                                        }
                                    }
                                }

                                Item {
                                    height: 16
                                    width: 1
                                }

                                FileMenuHeading {
                                    text: "Available"
                                }

                                Flow {
                                    id: flow
                                    anchors {
                                        left: parent.left
                                        right: parent.right
                                    }

                                    spacing: 16

                                    ProgressBar {
                                        id: communityProgressBar
                                        indeterminate: true
                                    }

                                    Repeater {
                                        id: communityRepeater
                                        delegate: StoreItem {
                                            name: modelData.name
                                            description: modelData.description
                                            imageUrl: modelData.screenshot.url
                                            onClicked: {
                                                console.log("Pushing", JSON.stringify(modelData))
                                                stackView.push(simulationComponent)
                                                stackView.currentItem.objectData = modelData
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    FileMenuItem {
                        name: "Upload"
                        component: Item {
                            Column {
                                id: uploadColumn
                                width: 420
                                height: 420
                                spacing: 8

                                Label {
                                    text: "Name:"
//                                    color: Material.color(Material.Grey)
                                }

                                TextField {
                                    id: uploadNameField
                                    anchors {
                                        left: parent.left
                                        right: parent.right
                                    }
                                    placeholderText: "e.g. 'Lateral inhibition'"
                                }

                                Label {
                                    text: "Description:"
                                }

                                TextField {
                                    id: uploadDescriptionField
                                    anchors {
                                        left: parent.left
                                        right: parent.right
                                    }

                                    placeholderText: "e.g. 'Shows network effects of lateral inhibition.'"
                                }

                                Row {
                                    anchors {
                                        right: parent.right
                                    }
                                    spacing: 16

                                    Button {
                                        text: qsTr("Cancel")
                                        onClicked: uploadMenu.close()
                                    }
                                    Button {
                                        text: qsTr("upload")
                                        onClicked: {
                                            var tempFolder = StandardPaths.writableLocation(StandardPaths.TempLocation)
                                            var stateFilename = tempFolder + "/simulation.nfy"
                                            var screenshotFilename = tempFolder + "/screenshot.png"

                                            neuronify.fileManager.saveState(stateFilename)
                                            neuronify.saveScreenshot(screenshotFilename, function() {
                                                var data = neuronify.fileManager.serializeState()
                                                parse.upload("simulation.nfy", data, function(simulationFile) {
                                                    _downloadManager.upload(
                                                                screenshotFilename,
                                                                parse.serverUrl + "files/screenshot.png",
                                                                function(screenshotResult) {
                                                                    var screenshotFile = JSON.parse(screenshotResult)
                                                                    var simulation = {
                                                                        name: uploadNameField.text,
                                                                        description: uploadDescriptionField.text,
                                                                        simulation: {
                                                                            name: simulationFile.name,
                                                                            url: simulationFile.url,
                                                                            __type: "File"
                                                                        },
                                                                        screenshot: {
                                                                            name: screenshotFile.name,
                                                                            url: screenshotFile.url,
                                                                            __type: "File"
                                                                        }
                                                                    }
                                                                    if(parse.objectId) {
                                                                        simulation["owner"] = {
                                                                            __type: "Pointer",
                                                                            className: "_User",
                                                                            objectId: parse.objectId
                                                                        }
                                                                    }
                                                                    parse.post("Simulation", simulation)
                                                                    ToolTip.show("Upload successful!", 2000)
                                                                    uploadMenu.close()
                                                                })
                                                })
                                            })

                                        }
                                    }
                                }
                            }
                        }
                    }

                    FileMenuItem {
                        name: "Account"
                        component: Item {
                            Column {
                                Button {
                                    Material.theme: Material.Light
                                    visible: !parse.loggedIn
                                    width: 120
                                    text: "Sign up"
                                    onClicked: {
                                        parse.post("_User", {"username":"cooldude6","password":"p_n7!-e8","phone":"415-392-0202"})
                                    }
                                }

                                Button {
                                    Material.theme: Material.Light
                                    visible: !parse.loggedIn
                                    width: 120
                                    text: "Log in"
                                    onClicked: {
                                        parse.login("cooldude6", "p_n7!-e8")
                                    }
                                }

                                Button {
                                    Material.theme: Material.Light
                                    visible: parse.loggedIn
                                    width: 120
                                    text: "Log out"
                                    onClicked: {
                                        parse.logout()
                                    }
                                }
                            }
                        }
                    }

                    FileMenuItem {
                        name: "Options"
                        component: Item {}
                    }
                }
            }

            Item {
                id: titleRow
                anchors {
                    top: fileViewMenu.top
                    left: fileViewMenu.right
                    leftMargin: 48
                }

                height: fileViewTitle.height
                MouseArea {
                    id: stackBackButton
                    anchors {
                        left: parent.left
                        top: parent.top
                        bottom: parent.bottom
                    }
                    width: height

                    onClicked: {
                        stackView.pop(null)
                    }

                    MaterialIcon {
                        anchors {
                            fill: parent
                            margins: 6
                        }
                        name: "arrow_back"
                        color: "white"
                    }
                }

                Text {
                    id: fileViewTitle
                    anchors {
                        top: parent.top
                        left: stackBackButton.right
                        leftMargin: 8
                    }

                    color: "white"
                    font.pixelSize: 48
                    font.weight: Font.Light
                    text: viewColumn.currentName
                }
            }

            StackView {
                id: stackView
                anchors {
                    left: titleRow.left
                    top: titleRow.bottom
                    right: parent.right
                    bottom: parent.bottom
                    topMargin: 32
                    rightMargin: 0
                }
                clip: true

                state: "top"

                states: [
                    State {
                        name: "top"
                        when: stackView.depth < 2
                        AnchorChanges {
                            target: fileViewTitle
                            anchors.left: parent.left
                        }
                        PropertyChanges {
                            target: fileViewTitle
                            anchors.leftMargin: 0
                        }
                        PropertyChanges {
                            target: stackBackButton
                            opacity: 0.0
                        }
                    }
                ]
                transitions: [
                    Transition {
                        to: "top"
                        reversible: true
                        SequentialAnimation {
                            NumberAnimation {
                                property: "opacity"
                                duration: 300
                                easing.type: Easing.InOutQuad
                            }
                            AnchorAnimation {
                                duration: 600
                                easing.type: Easing.InOutQuad
                            }
                        }
                    }
                ]
            }
        }
        states: [
            State {
                name: "hidden"
                PropertyChanges {
                    target: fileViewContent
                    visible: false
                }
                PropertyChanges {
                    target: titleRow
                    opacity: 0.0
                }
                PropertyChanges {
                    target: titleRow
                    anchors.leftMargin: 1024
                }
                PropertyChanges {
                    target: fileViewMenu
                    opacity: 0.0
                }
                AnchorChanges {
                    target: viewColumn
                    anchors {
                        left: undefined
                        right: parent.left
                    }
                }
            }
        ]

        transitions: [
            Transition {
                to: ""
                NumberAnimation {
                    targets: [titleRow, fileViewMenu]
                    properties: "opacity"
                    duration: 600
                    easing.type: Easing.InOutQuad
                }
                NumberAnimation {
                    targets: [titleRow]
                    properties: "anchors.leftMargin"
                    duration: 360
                    easing.type: Easing.OutQuad
                }
                AnchorAnimation {
                    targets: viewColumn
                    duration: 400
                    easing.type: Easing.InOutQuad
                }
            }
        ]
    }

    HudShadow {
        id: leftMenuShadow
        anchors.fill: leftMenu
        source: leftMenu
        z: 38
    }

    Item {
        id: itemMenu

        anchors {
            left: leftMenu.right
            top: parent.top
            topMargin: 64
            bottom: parent.bottom
            bottomMargin: 64
            //            bottomMargin: 120
        }

        width: 280 + 32
        height: itemColumn.height
        z: 20

        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            onWheel: {
                // NOTE: necessary to capture wheel events
            }
        }

        ListModel {
            id: categories
            ListElement {
                listSource: "qrc:/qml/hud/NeuronList.qml"
                imageSource: "qrc:/images/categories/excitatory.png"
                text: "Excitatory neurons"
            }
            ListElement  {
                listSource: "qrc:/qml/hud/InhibitoryNeuronList.qml"
                imageSource: "qrc:/images/categories/inhibitory.png"
                text: "Inhibitory neurons"
            }

            ListElement  {
                listSource: "qrc:/qml/hud/MetersList.qml"
                imageSource: "qrc:/images/categories/meters.png"
                text: "Measurement devices"
            }

            ListElement  {
                listSource: "qrc:/qml/hud/GeneratorsList.qml"
                imageSource: "qrc:/images/categories/generators.png"
                text: "Generators"
            }
            ListElement  {
                listSource: "qrc:/qml/hud/AnnotationsList.qml"
                imageSource: "qrc:/images/categories/annotation.png"
                text: "Annotation"
            }
        }

        Rectangle {
            id: itemMenuBackground
            color: "#e3eef9"
            anchors {
                fill: itemFlickable
                topMargin: -16
                bottomMargin: -16
            }
        }

        HudShadow {
            id: itemMenuShadow
            anchors.fill: itemMenuBackground
            source: itemMenuBackground
        }

        Flickable {
            id: itemFlickable
            anchors {
                left: parent.left
                right: parent.right
            }

            height: Math.min(parent.height, itemColumn.height)
            clip: true

            //            ScrollIndicator.vertical: ScrollIndicator {}
            ScrollBar.vertical: ScrollBar {}
            contentHeight: itemColumn.height
            //            interactive: false

            Column {
                id: itemColumn
                property int currentIndex: -1

                anchors {
                    left: parent.left
                    right: parent.right
                }

                Component.onCompleted: {
                    currentIndex = 0
                }

                Repeater {
                    model: categories
                    Column {
                        anchors {
                            left: parent.left
                            right: parent.right
                        }
                        spacing: 12
                        Text {
                            anchors {
                                left: parent.left
                                right: parent.right
                                margins: 16
                            }
                            font.pixelSize: 18
                            font.family: Style.font.family
                            color: Style.mainDesktop.text.color
                            wrapMode: Text.WrapAtWordBoundaryOrAnywhere
                            text: model.text
                        }

                        Flow {
                            id: itemListView
                            property int currentIndex: 0
                            property alias listSource: itemModelLoader.source
                            property int rows: Math.floor(parent.height / 96)
                            property int columns: 3
                            property real itemHeight: (height - spacing * (rows - 1)) / rows
                            property real itemWidth: (width - spacing * (columns - 1)) / columns

                            anchors {
                                left: parent.left
                                right: parent.right
                                leftMargin: 24
                                rightMargin: 48
                            }

                            spacing: 8

                            Loader {
                                id: itemModelLoader
                                source: model.listSource
                            }

                            Repeater {
                                id: itemListRepeater

                                model: itemModelLoader.item

                                CreationItem {
                                    id: creationItem

                                    //                                    width: itemListView.itemWidth
                                    width: itemListView.itemWidth

                                    parentWhenDragging: root

                                    name: model.name
                                    description: model.description
                                    source: model.source
                                    imageSource: model.imageSource

                                    onDragActiveChanged: {
                                        if(dragActive) {
                                            root.dragging = true
                                        } else {
                                            root.dragging = false
                                        }
                                        showInfoPanelTimer.stop()
                                    }

                                    MouseArea {
                                        id: hoverArea
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        acceptedButtons: Qt.NoButton
                                        propagateComposedEvents: true
                                        onEntered: {
                                            infoPanel.selectedItem = creationItem
                                            hideInfoPanelTimer.stop()
                                            showInfoPanelTimer.restart()
                                        }
                                        onExited: {
                                            hideInfoPanelTimer.restart()
                                            showInfoPanelTimer.stop()
                                        }
                                    }

                                    Timer {
                                        id: showInfoPanelTimer
                                        interval: 400
                                        onTriggered: {
                                            if(hoverArea.containsMouse) {
                                                infoPanel.state = "revealed"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        states: [
            State {
                name: "dragging"
                when: root.dragging
                PropertyChanges {
                    target: itemMenu
                    opacity: 0.0
                }
            },
            State {
                name: "hidden"
                PropertyChanges { target: itemMenu; anchors.leftMargin: -itemMenu.width }
            }
        ]

        transitions: [
            Transition {
                NumberAnimation {
                    properties: "opacity"
                    duration: 200
                }
                NumberAnimation {
                    properties: "anchors.leftMargin"
                    duration: 400
                    easing.type: Easing.InOutQuad
                }
            }
        ]
    }

    Item {
        id: savePanel

        anchors {
            left: leftMenu.right
            top: parent.top
            topMargin: 64
            bottom: parent.bottom
            bottomMargin: 64
            //            bottomMargin: 120
        }

        width: 160
        //        height: savePanelColumn.height
        z: 20

        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            onWheel: {
                // NOTE: necessary to capture wheel events
            }
        }

        Rectangle {
            id: savePanelBackground
            color: "#e3eef9"
            anchors {
                fill: savePanelFlickable
                topMargin: -16
                bottomMargin: -16
            }
        }

        HudShadow {
            id: savePanelShadow
            anchors.fill: savePanelBackground
            source: savePanelBackground
        }

        Flickable {
            id: savePanelFlickable
            anchors {
                left: parent.left
                right: parent.right
            }

            height: Math.min(parent.height, savePanelColumn.height)
            clip: true

            //            ScrollIndicator.vertical: ScrollIndicator {}
            ScrollBar.vertical: ScrollBar {}
            contentHeight: savePanelColumn.height
            //            interactive: false

            Column {
                id: savePanelColumn
                property int currentIndex: -1

                anchors {
                    left: parent.left
                    leftMargin: 32
                    right: parent.right
                }

                Component.onCompleted: {
                    currentIndex = 0
                }

                Button {
                    text: qsTr("Save")
                    onClicked: {
                        // TODO implement save
                        ToolTip.show("TODO: Implement quicksave")
                    }
                }

                Button {
                    text: qsTr("Save as")
                    onClicked: {
                        root.state = "welcome"
                        viewColumn.openSave()
                    }
                }

                Button {
                    text: qsTr("Upload")
                    onClicked: {
                        viewColumn.openSave()
                    }
                }
            }
        }

        states: [
            State {
                name: "hidden"
                PropertyChanges { target: savePanel; anchors.leftMargin: -savePanel.width }
            }
        ]

        transitions: [
            Transition {
                NumberAnimation {
                    properties: "opacity"
                    duration: 200
                }
                NumberAnimation {
                    properties: "anchors.leftMargin"
                    duration: 400
                    easing.type: Easing.InOutQuad
                }
            }
        ]
    }

    Item {
        id: infoPanel

        property var selectedItem

        anchors {
            left: itemMenu.right
            leftMargin: 0
            top: itemMenu.top
            topMargin: 12

            Behavior on topMargin {
                SmoothedAnimation {
                    duration: 400
                    easing.type: Easing.InOutQuad
                }
            }
        }

        state: "hidden"

        width: 240
        height: infoColumn.height + infoColumn.anchors.margins * 2

        Rectangle {
            id: infoBackground
            anchors.fill: parent
            visible: false
            color: "#fafafa"
        }

        HudShadow {
            anchors.fill: infoBackground
            source: infoBackground
        }

        Column {
            id: infoColumn
            anchors {
                left: parent.left
                right: parent.right
                top: parent.top
                margins: 16
                leftMargin: 20
            }
            spacing: 8
            Text {
                anchors {
                    left: parent.left
                    right: parent.right
                }
                wrapMode: Text.WrapAtWordBoundaryOrAnywhere
                font.pixelSize: 18
                color: "#676767"
                text: infoPanel.selectedItem ? infoPanel.selectedItem.name : "Nothing selected"
            }
            Text {
                anchors {
                    left: parent.left
                    right: parent.right
                }

                wrapMode: Text.WrapAtWordBoundaryOrAnywhere
                font.pixelSize: 14
                text: infoPanel.selectedItem ? infoPanel.selectedItem.description : "Nothing selected"
            }
        }

        Timer {
            id: hideInfoPanelTimer
            interval: 1000
            onTriggered: {
                infoPanel.state = "hidden"
            }
        }

        states: [
            State {
                name: "hidden"
                PropertyChanges {
                    target: infoPanel; anchors.leftMargin: -width
                }
            },
            State {
                name: "revealed"
            },
            State {
                name: "dragging"
                extend: "hidden"
                when: root.dragging
                onCompleted: infoPanel.state = "hidden"
                PropertyChanges {
                    target: infoPanel
                    opacity: 0.0
                }
            }
        ]

        transitions: [
            Transition {
                NumberAnimation {
                    properties: "anchors.leftMargin"
                    duration: 800
                    easing.type: Easing.InOutQuad
                }
                NumberAnimation {
                    properties: "opacity"
                    duration: 200
                }
            }
        ]
    }

    states: [
        State {
            name: "view"
            PropertyChanges { target: fileView; state: "hidden" }
            PropertyChanges { target: infoPanel; state: "hidden" }
            PropertyChanges { target: itemMenu; state: "hidden" }
            PropertyChanges { target: savePanel; state: "hidden" }
        },
        State {
            name: "creation"
            extend: "view"
            PropertyChanges { target: leftMenu; state: "small" }
            PropertyChanges { target: itemMenu; state: "" }
        },
        State {
            name: "welcome"
            extend: "view"
            PropertyChanges { target: fileView; state: "" }
            PropertyChanges { target: leftMenuShadow; opacity: 0.0 }
            PropertyChanges { target: leftMenu; state: "hidden" }
        },
        State {
            name: "save"
            extend: "view"
            PropertyChanges { target: savePanel; state: "" }
        },
        State {
            name: "projects"
            extend: "view"
        },
        State {
            name: "help"
            extend: "view"
        }

    ]

    transitions: [
        Transition {
            animations: [
                animateCreation,
                animateLeftMenu,
            ]
        },
        Transition {
            to: "community"
            animations: [
                animateLeftMenu,
                animateCreation,
                animateCommunityTextIn
            ]
        },
        Transition {
            from: "community"
            animations: [
                animateLeftMenu,
                animateCreation,
                animateCommunityTextOut
            ]
        }
    ]

    ParallelAnimation {
        id: animateLeftMenu
        NumberAnimation {
            target: logoTextCopy
            property: "opacity"
            duration: 400
            easing.type: Easing.InOutQuad
        }
    }

    ParallelAnimation {
        id: animateCreation
        NumberAnimation {
            properties: "anchors.leftMargin"
            duration: 400
            easing.type: Easing.InOutQuad
        }
        ColorAnimation {
            properties: "color"
            duration: 400
            easing.type: Easing.InOutQuad
        }
    }

    SequentialAnimation {
        id: animateCommunityTextIn
        PauseAnimation {
            duration: 400
        }
    }
    SequentialAnimation {
        id: animateCommunityTextOut
        PauseAnimation {
            duration: 200
        }
    }
}
