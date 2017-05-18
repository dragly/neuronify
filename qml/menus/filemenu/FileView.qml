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
    id: fileView
    
    Material.theme: Material.Dark
    
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
            color: topMenu.color
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
        
        Item {
            id: fileViewMenu
            anchors {
                left: parent.left
                top: parent.top
                topMargin: 64
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
                                        
                                        onClicked: {
                                            neuronify.loadSimulation(loader.item.stateSource)
                                            root.state = "view"
                                        }
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
                                    Material.theme: Material.Light
                                    text: qsTr("Cancel")
                                    onClicked: uploadMenu.close()
                                }
                                Button {
                                    Material.theme: Material.Light
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
                target: fileView
                enabled: false
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
            //                to: ""
            NumberAnimation {
                targets: [titleRow, fileViewMenu]
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
        //            Transition {
        //                to: "hidden"
        
        //            }
    ]
}
