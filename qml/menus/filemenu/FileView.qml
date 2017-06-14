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

    property bool revealed: true
    property url latestFolder: StandardPaths.writableLocation(StandardPaths.DocumentsLocation) + "/neuronify"
    property var currentSimulation

    signal saveRequested(var simulation)
    signal openRequested(url file)
    signal runRequested(var simulation)
    signal loadRequested(url file) // TODO remove

    onRevealedChanged: console.log("REVEALED", revealed)
    
    Material.theme: Material.Dark

    function open(name) {
        var index = 0
        for(var i in viewColumn.children) {
            var child = viewColumn.children[i]
            if(child.identifier === name) {
                viewColumn.currentIndex = index
                break
            }
            index += 1
        }
        root.revealed = true
    }

    Settings {
        property alias latestFolder: root.latestFolder
    }
    
    MouseArea {
        id: fileViewMouseArea
        anchors.fill: parent
        onWheel: {
            return
        }
    }
    
    Item {
        id: fileViewContent
        anchors.fill: parent
        
        Rectangle {
            id: background
            anchors.fill: parent
            color: Material.primary
            opacity: 1.0
        }
        
        //        ShaderEffectSource {
        //            id: neuronifySource
        //            anchors.fill: parent
        //            visible: false
        //            sourceItem: neuronify.shaderEffectItem
        //        }
        
        //        GaussianBlur {
        //            id: blur
        //            anchors.fill: parent
        //            radius: 48
        //            samples: 64
        //            source: neuronifySource
        //            opacity: 0.2
        //        }
        
        Item {
            id: fileViewMenu
            anchors {
                left: parent.left
                leftMargin: 48
                top: parent.top
                topMargin: 64
            }
            width: 196
            height: buttonContainer.height
            
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

            Column {
                id: buttonContainer
                anchors {
                    left: parent.left
                    right: parent.right
                }

                FileMenuItem {
                    name: "Back"
                    onClicked: {
                        root.revealed = false
                    }
                }

                Item {
                    height: 32
                    width: 32
                }

                FileMenu {
                    id: viewColumn
                    property string currentName
                    currentIndex: 0

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
                        identifier: "new"
                        name: "New"
                        component: Flickable {
                            contentHeight: newColumn.height + 64
                            clip: true

                            flickableDirection: Flickable.VerticalFlick
                            ScrollBar.vertical: ScrollBar {}
                            Column {
                                id: newColumn
                                anchors {
                                    left: parent.left
                                    right: parent.right
                                    rightMargin: 16
                                }

                                spacing: 32

                                StoreItem {
                                    name: "New simulation"
                                    description: "A blank canvas."
                                    onClicked: {
                                        loadRequested("qrc:/simulations/empty/empty.nfy")
                                    }
                                }

                                // TODO replace with database
                                Repeater {
                                    model: [
                                        {
                                            name: "Tutorial",
                                            simulations: [
                                                "qrc:/simulations/tutorial/tutorial_1_intro",
                                                "qrc:/simulations/tutorial/tutorial_2_circuits",
                                                "qrc:/simulations/tutorial/tutorial_3_creation",
                                            ]
                                        },
                                        {
                                            name: "Neuronify Items",
                                            simulations: [
                                                "qrc:/simulations/items/neurons/leaky",
                                                "qrc:/simulations/items/neurons/inhibitory",
                                                "qrc:/simulations/items/neurons/adaptation",
                                                "qrc:/simulations/items/visualInput",
                                                "qrc:/simulations/items/generators",
                                                "qrc:/simulations/items/frPlot",

                                            ]
                                        },
                                        {
                                            name: "Miscellaneous",
                                            simulations: [
                                                "qrc:/simulations/mix/lateral_inhibition",
                                                "qrc:/simulations/mix/recurrent_inhibition",
                                                "qrc:/simulations/mix/reciprocal_inhibition",
                                                "qrc:/simulations/mix/disinhibition",
                                                "qrc:/simulations/mix/rythm_transformation",
                                                "qrc:/simulations/mix/prolonged_activity",
                                            ]
                                        },
                                        {
                                            name: "Textbook Examples",
                                            simulations: [
                                                "qrc:/simulations/mix/lateral_inhibition_1",
                                                "qrc:/simulations/mix/lateral_inhibition_2",
                                                "qrc:/simulations/mix/input_summation",
                                                "qrc:/simulations/sterratt/if_response",
                                                "qrc:/simulations/sterratt/refractory_period",
                                            ]
                                        },
                                    ]

                                    Column {

                                        anchors {
                                            left: parent.left
                                            right: parent.right
                                        }

                                        spacing: 16

                                        Label {
                                            font.pixelSize: 24
                                            text: modelData.name
                                        }

                                        Flow {
                                            anchors {
                                                left: parent.left
                                                right: parent.right
                                            }
                                            spacing: 32
                                            Repeater {
                                                model: modelData.simulations
                                                StoreItem {
                                                    SimulationLoader {
                                                        id: loader
                                                        folder: modelData
                                                    }

                                                    name: loader.item.name
                                                    description: loader.item.description
                                                    imageUrl: loader.item.screenshotSource

                                                    onClicked: {
                                                        loadRequested(loader.item.stateSource)
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    FileMenuItem {
                        identifier: "open"
                        name: "Open"
                        component: Item {
                            id: openItem

                            Settings {
                                id: savedataSettings
                                property bool performed
                                property url location
                                category: "converted_saves"
                            }

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
                                        openDialog.open()
                                    }
                                }

                                Column {
                                    anchors {
                                        left: parent.left
                                        right: parent.right
                                    }
                                    visible: savedataSettings.performed
                                    Button {
                                        Material.theme: Material.Light
                                        text: "From older versions"
                                        onClicked: {
                                            openDialog.open()
                                            console.log("Opening", savedataSettings.location)
                                            openDialog.folder = savedataSettings.location
                                        }
                                    }
                                }

                                FileDialog {
                                    id: openDialog
                                    fileMode: FileDialog.OpenFile
                                    nameFilters: ["Neuronify files (*.neuronify)"]
                                    onAccepted: {
                                        openRequested(file)
                                    }
                                }

                                // TODO Add recent once database is ready

                                //                                FileMenuHeading {
                                //                                    text: "Recent"
                                //                                }

                                //                                Flow {
                                //                                    anchors {
                                //                                        left: parent.left
                                //                                        right: parent.right
                                //                                    }
                                //                                }
                            }
                        }
                    }

                    FileMenuItem {
                        identifier: "save"
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
                                        regExp: /[a-zA-Z0-9\-\_ ]+/
                                    }

                                    placeholderText: "e.g. MySimulation"

                                    Binding {
                                        target: saveName
                                        property: "text"
                                        when: currentSimulation ? true : false
                                        value: currentSimulation ? currentSimulation.name : ""
                                    }
                                }

                                Label {
                                    text: qsTr("Description:")
                                }

                                TextArea {
                                    id: saveDescription
                                    anchors {
                                        left: parent.left
                                        right: parent.right
                                    }
                                    placeholderText: "e.g. Illustrates the effect of feedback inhibition."

                                    Binding {
                                        target: saveDescription
                                        property: "text"
                                        when: currentSimulation ? true : false
                                        value: currentSimulation ? currentSimulation.description : ""
                                    }
                                }

                                Label {
                                    text: "Screenshot preview:"
                                }

                                ShaderEffectSource {
                                    id: effectSource
                                    readonly property real aspectRatio: neuronify.width / neuronify.height
                                    width: parent.width * 0.6
                                    height: width / aspectRatio
                                    sourceItem: neuronify
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
                                        saveDialog.open()
                                    }
                                }

                                FileDialog {
                                    id: saveDialog
                                    fileMode: FileDialog.SaveFile
                                    nameFilters: ["Neuronify files (*.neuronify)"]
                                    defaultSuffix: ".neuronify"
                                    onAccepted: {
                                        var simulation = {
                                            file: file,
                                            name: saveName.text,
                                            description: saveDescription.text
                                        }

                                        saveRequested(simulation)
                                    }
                                }
                            }
                        }

                    }
                    FileMenuItem {
                        identifier: "community"
                        name: "Community"
                        component: Flickable {
                            contentHeight: column.height + 64
                            clip: true

                            flickableDirection: Flickable.VerticalFlick
                            ScrollBar.vertical: ScrollBar {}

                            Column {
                                id: column
                                anchors {
                                    left: parent.left
                                    right: parent.right
                                }

                                spacing: 16

                                Component.onCompleted: {
                                    communityProgressBar.processCount += 1
                                    Parse.get('Tag?where={"objectId":{"$in":["KoorVWOOtU"]}}&order=-priority', function(response) {
                                        communityProgressBar.processCount -= 1
                                        communityTagsRepeater.model = response.results
                                    })
                                }

                                Label {
                                    anchors {
                                        left: parent.left
                                        right: parent.right
                                    }

                                    wrapMode: Label.WrapAtWordBoundaryOrAnywhere
                                    onLinkActivated: Qt.openUrlExternally(link)
                                    linkColor: "white"

                                    text: "Would you like to see your simulations listed here? " +
                                          "Send an e-mail to <a href='mailto:ovilab.net@gmail.com'>ovilab.net@gmail.com</a> " +
                                          "to request upload rights."
                                }

                                ProgressBar {
                                    id: communityProgressBar

                                    property int processCount: 0

                                    indeterminate: true
                                    visible: processCount > 0
                                }

                                Repeater {
                                    id: communityTagsRepeater

                                    Column {
                                        id: tag

                                        anchors {
                                            left: parent.left
                                            right: parent.right
                                        }
                                        spacing: 16

                                        Component.onCompleted: {
                                            communityProgressBar.processCount += 1
                                            Parse.get('Simulation?where={"tags":{"__type":"Pointer","className":"Tag","objectId":"' + modelData.objectId + '"}}', function(response) {
                                                communityProgressBar.processCount -= 1
                                                communityRepeater.model = response.results
                                            })
                                        }

                                        Label {
                                            font.pixelSize: 24
                                            text: modelData.name
                                        }

                                        FolderListModel {
                                            id: communityFolderModel
                                            folder: StandardPaths.writableLocation(StandardPaths.AppDataLocation) + "/community"
                                            showFiles: false
                                            showDirs: true
                                            showOnlyReadable: true
                                        }

                                        Component {
                                            id: simulationComponent
                                            StoreSimulation {
                                                downloadManager: _downloadManager // TODO is this needed anymore?
                                                onRunClicked: {
                                                    //                                        loadRequested(fileUrl)
                                                    //                                        openRequested(fileUrl)
                                                    runRequested(simulation)
                                                    stackView.pop()
                                                }
                                            }
                                        }

                                        Flow {
                                            id: flow
                                            anchors {
                                                left: parent.left
                                                right: parent.right
                                            }

                                            spacing: 16

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
                        }
                    }

                    FileMenuItem {
                        name: "Upload"
                        visible: Parse.loggedIn
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

                                Label {
                                    text: "Screenshot preview:"
                                }

                                ShaderEffectSource {
                                    readonly property real aspectRatio: neuronify.width / neuronify.height
                                    width: parent.width * 0.6
                                    height: width / aspectRatio
                                    sourceItem: neuronify
                                }

                                Row {
                                    anchors {
                                        right: parent.right
                                    }
                                    spacing: 16
                                    Button {
                                        id: uploadButton
                                        Material.theme: Material.Light
                                        text: "Upload"
                                        onClicked: {
                                            uploadButton.enabled = false
                                            var tempFolder = StandardPaths.writableLocation(StandardPaths.TempLocation)
                                            var screenshotFilename = tempFolder + "/screenshot.png"
                                            neuronify.saveScreenshot(screenshotFilename, function() {
                                                var data = neuronify.fileManager.serializeState()
                                                Parse.upload("simulation.nfy", data, function(simulationFile) {
                                                    _downloadManager.upload(
                                                                screenshotFilename,
                                                                Parse.serverUrl + "files/screenshot.png",
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
                                                                    if(Parse.objectId) {
                                                                        simulation["owner"] = {
                                                                            __type: "Pointer",
                                                                            className: "_User",
                                                                            objectId: Parse.objectId
                                                                        }
                                                                    }
                                                                    Parse.post("Simulation", simulation)
                                                                    ToolTip.show("Upload successful!", 2000)
                                                                    root.revealed = false
                                                                    uploadButton.enabled = true
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
                        identifier: "account"
                        name: "Account"
                        component: Item {
                            function login() {
                                Parse.login(userField.text, passwordField.text, function(response) {
                                    if(!response.sessionToken) {
                                        passwordField.ToolTip.show("Wrong username or password", 1000)
                                    }
                                })
                            }

                            Column {
                                visible: !Parse.loggedIn
                                TextField {
                                    id: userField
                                    selectByMouse: true
                                    placeholderText: "Username or email"
                                }

                                TextField {
                                    id: passwordField
                                    echoMode: TextInput.Password
                                    selectByMouse: true
                                    placeholderText: "Password"
                                    onAccepted: {
                                        login()
                                    }
                                }

                                Button {
                                    Material.theme: Material.Light
                                    enabled: userField.text != "" && passwordField.text != ""
                                    width: 120
                                    text: "Log in"
                                    onClicked: {
                                        login()
                                    }
                                }
                            }

                            Button {
                                Material.theme: Material.Light
                                visible: Parse.loggedIn
                                width: 120
                                text: "Log out"
                                onClicked: {
                                    Parse.logout()
                                }
                            }
                        }
                    }


                    // TODO add back settings view when ready
                    //                    FileMenuItem {
                    //                        identifier: "settings"
                    //                        name: "Settings"
                    //                        component: Item {}
                    //                    }
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

            replaceEnter: Transition {
                ParallelAnimation {
                    XAnimator {
                        from: 400
                        to: 0
                        duration: 400
                        easing.type: Easing.OutCubic
                    }
                    PropertyAnimation {
                        property: "opacity"
                        from: 0
                        to: 1
                        duration: 400
                        easing.type: Easing.OutCubic
                    }
                }
            }

            replaceExit: Transition {
                ParallelAnimation {
                    XAnimator {
                        from: 0
                        to: -400
                        duration: 400
                        easing.type: Easing.OutCubic
                    }
                    PropertyAnimation {
                        property: "opacity"
                        from: 1
                        to: 0
                        duration: 400
                        easing.type: Easing.OutCubic
                    }
                }
            }

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
            when: !root.revealed
            PropertyChanges {
                target: root
                enabled: false
                opacity: 0.0
            }
            PropertyChanges {
                target: fileViewContent
                opacity: 0.0
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
                target: stackView
                anchors.rightMargin: -1024
            }
            PropertyChanges {
                target: fileViewMenu
                opacity: 0.0
            }
            PropertyChanges {
                target: fileViewMouseArea
                enabled: false
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
            NumberAnimation {
                targets: [titleRow, fileViewMenu]
                properties: "opacity"
                duration: 600
                easing.type: Easing.InOutQuad
            }
            NumberAnimation {
                targets: root
                properties: "opacity"
                duration: 600
                easing.type: Easing.InOutQuad
            }
            NumberAnimation {
                properties: "anchors.leftMargin,anchors.rightMargin"
                duration: 360
                easing.type: Easing.OutQuad
            }
            AnchorAnimation {
                targets: viewColumn
                duration: 400
                easing.type: Easing.InOutQuad
            }
            NumberAnimation {
                target: fileViewContent
                properties: "opacity"
                duration: 400
            }
        }
    ]
}
