import QtQuick 2.5
import QtQuick.Controls 2.0
import QtQuick.Dialogs 1.0
import QtQuick.Layouts 1.3
import QtQuick.Particles 2.0
import QtQuick.Window 2.1

import QtCharts 2.1
import QtMultimedia 5.5
import Qt.labs.settings 1.0

import Neuronify 1.0
import CuteVersioning 1.0
import QtGraphicalEffects 1.0

import "qrc:/qml/backend"
import "qrc:/qml/controls"
import "qrc:/qml/hud"
import "qrc:/qml/io"
import "qrc:/qml/menus/mainmenu"
import "qrc:/qml/tools"
import "qrc:/qml/store"
import "qrc:/qml/style"

Item {
    id: root

    property bool dragging: false

    state: "save"

    DownloadManager {
        id: downloadManager
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

        color: "#6BAED6"
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
                topMargin: 24
            }
            spacing: 24
            Repeater {
                model: ListModel {
                    ListElement {
                        state: "welcome"
                        name: "Welcome"
                    }
                    ListElement {
                        state: "view"
                        name: "View"
                    }
                    ListElement {
                        state: "creation"
                        name: "Create"
                    }
                    ListElement {
                        state: "save"
                        name: "Save"
                    }
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

        Column {
            anchors {
                bottom: parent.bottom
            }

            Button {
                visible: !parse.loggedIn
                width: 120
                text: "Sign up"
                onClicked: {
                    parse.post("_User", {"username":"cooldude6","password":"p_n7!-e8","phone":"415-392-0202"})
                }
            }

            Button {
                visible: !parse.loggedIn
                width: 120
                text: "Log in"
                onClicked: {
                    parse.login("cooldude6", "p_n7!-e8")
                }
            }

            Button {
                visible: parse.loggedIn
                text: "Log out"
                onClicked: {
                    parse.logout()
                }
            }

        }

        onStateChanged: console.log("Left menu state", state)

        states: [
            State {
                name: "small"
                PropertyChanges { target: leftMenu; width: 72 }
                PropertyChanges { target: logoTextCopy; opacity: 0.0 }
                PropertyChanges { target: leftMenu; textOpacity: 0.0 }
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
            }
        ]
    }

    Item {
        id: communityView
        anchors {
            left: leftMenu.right
            top: parent.top
            bottom: parent.bottom
            right: parent.right
        }
        z: 39

        Rectangle {
            id: communityBackground
            anchors {
                left: parent.left
                top: parent.top
                bottom: parent.bottom
            }
            width: parent.width
            color: leftMenu.color
            state: "hidden"

            Item {
                id: viewItem
                anchors {
                    left: parent.left
                    top: parent.top
                    margins: 64
                }
                width: 160
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

                Column {
                    id: viewColumn
                    property int currentIndex: 4
                    property var currentElement: viewModel.get(currentIndex)

                    anchors {
                        left: parent.left
                        right: parent.right
                    }

                    Repeater {
                        model: ListModel {
                            id: viewModel
                            ListElement {
                                name: "New"
                                state: "new"
                            }

                            ListElement {
                                name: "Open"
                                state: "open"
                            }

                            ListElement {
                                name: "Save"
                                state: "save"
                            }

                            ListElement {
                                name: "Examples"
                                state: "examples"
                            }

                            ListElement {
                                name: "Community"
                                state: "community"
                            }

                            //                        ListElement {
                            //                            name: "Plugins"
                            //                        }
                        }

                        delegate: ItemDelegate {

                            id: rectangle4
                            anchors {
                                left: parent.left
                                right: parent.right
                            }

                            height: 56
                            highlighted: viewColumn.currentIndex === index

                            onClicked: {
                                viewColumn.currentIndex = index
                            }

                            Label {
                                id: text2
                                anchors {
                                    left: parent.left
                                    leftMargin: 16
                                    verticalCenter: parent.verticalCenter
                                }
                                color: "white"
                                font.pixelSize: 18
                                text: name
                            }
                        }
                    }
                }
            }

            StackLayout {
                id: stackView
                anchors {
                    left: viewItem.right
                    top: viewItem.top
                    right: parent.right
                    bottom: parent.bottom
                    leftMargin: 48
                    rightMargin: 48
                }

                currentIndex: viewColumn.currentIndex

                Item {
                    Text {
                        color: "white"
                        font.pixelSize: 48
                        font.weight: Font.Light
                        text: "New"
                    }
                }

                Item {
                    Text {
                        color: "white"
                        font.pixelSize: 48
                        font.weight: Font.Light
                        text: "Open"
                    }
                }

                Item {
                    Text {
                        color: "white"
                        font.pixelSize: 48
                        font.weight: Font.Light
                        text: "Save"
                    }
                }

                Item {
                    Text {
                        color: "white"
                        font.pixelSize: 48
                        font.weight: Font.Light
                        text: "Examples"
                    }
                }

                Item {
                    Item {
                        id: communityRow
                        height: communityTitle.height
                        MouseArea {
                            id: communityBack
                            anchors {
                                left: parent.left
                                top: parent.top
                                bottom: parent.bottom
                            }
                            width: height

                            onClicked: {
                                stackView2.pop()
                            }

                            Image {
                                anchors {
                                    fill: parent
                                    margins: 6
                                }
                                source: "qrc:/images/back.png"
                            }
                        }

                        Text {
                            id: communityTitle
                            anchors {
                                top: parent.top
                                left: communityBack.right
                            }

                            color: "white"
                            font.pixelSize: 48
                            font.weight: Font.Light
                            text: "Community"
                        }
                    }
                    StackView {
                        id: stackView2
                        anchors {
                            top: communityRow.bottom
                            left: parent.left
                            right: parent.right
                            bottom: parent.bottom
                            topMargin: 32
                        }
                        clip: true

                        initialItem: Flickable {
                            contentHeight: column.height
                            clip: true

                            flickableDirection: Flickable.VerticalFlick
                            ScrollBar.vertical: ScrollBar {}

                            Component.onCompleted: {
                                parse.get("Simulation", function(response) {
                                    for(var i in response.results) {
                                        var simulation = response.results[i]
                                        listModel.append(response.results[i])
                                    }
                                })
                            }

                            Column {
                                id: column
                                anchors {
                                    left: parent.left
                                    right: parent.right
                                }

                                spacing: 16

                                Text {
                                    color: "white"
                                    font.pixelSize: 24
                                    font.weight: Font.Light
                                    text: "Simulations"
                                }

                                Flow {
                                    id: flow
                                    anchors {
                                        left: parent.left
                                        right: parent.right
                                    }

                                    spacing: 16

                                    Repeater {
                                        model: ListModel { id: listModel }
                                        delegate: StoreItem {
                                            width: 160
                                            height: 256
                                            name: model.name
                                            description: model.description ? model.description : ""
                                            imageUrl: model.screenshot ? model.screenshot.url : ""
                                            onClicked: {
                                                console.log("Pushing")
                                                stackView2.push(simulationComponent)
                                                stackView2.currentItem.objectId = model.objectId
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Component {
                        id: simulationComponent
                        StoreSimulation {
                            backend: parse
                            onDownloadClicked: {
                                var targetLocation = StandardPaths.writableLocation(
                                            StandardPaths.AppDataLocation,
                                            "examples/" + objectData.objectId
                                            )
                                if(objectData.simulation) {
                                    downloadManager.download(
                                                objectData.simulation.url,
                                                targetLocation,
                                                "simulation.nfy")
                                }
                                if(objectData.screenshot) {
                                    downloadManager.download(
                                                objectData.screenshot.url,
                                                targetLocation,
                                                "screenshot.png")
                                }
                            }
                        }
                    }

                    state: "top"

                    states: [
                        State {
                            name: "top"
                            when: stackView2.depth < 2
                            PropertyChanges {
                                target: communityBack
                                anchors.leftMargin: -communityBack.width
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
                                NumberAnimation {
                                    property: "anchors.leftMargin"
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
                        target: communityBackground
                        anchors.leftMargin: -width
                    }
                }
            ]

            transitions: [
                Transition {
                    NumberAnimation {
                        duration: 400
                        properties: "anchors.leftMargin"
                        easing.type: Easing.OutQuad
                    }
                }
            ]
        }
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
                    }
                }

                Button {
                    text: qsTr("Save as")
                    onClicked: {
                        saveMenu.open()
                    }
                }

                Button {
                    text: qsTr("Upload")
                    onClicked: {
                        uploadMenu.open()
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

    Popup {
        id: saveMenu

        x: parent.width / 2 - width / 2
        y: parent.height / 2 - height / 2

        modal: true
        padding: 32

        Column {
            id: saveColumn
            width: 420
            height: 420
            spacing: 8

            Label {
                text: "Name:"
            }

            TextField {
                id: nameField
                anchors {
                    left: parent.left
                    right: parent.right
                }

                placeholderText: "Name"
            }

            Label {
                text: "Location:"
            }

            RowLayout {
                anchors {
                    left: parent.left
                    right: parent.right
                }
                spacing: 8

                TextField {
                    id: locationField
                    Layout.fillWidth: true
                    text: StandardPaths.writableLocation(StandardPaths.DocumentsLocation, "neuronify")
                }

                Button {
                    text: "Browse"
                }
            }

            Row {
                Button {
                    text: qsTr("Cancel")
                    onClicked: saveMenu.close()
                }
                Button {
                    text: qsTr("Save")
                    onClicked: {
                        if(!neuronify.saveState("file://" + locationField.text + "/" + nameField.text + ".nfy")) {
                            ToolTip.show("Error: Could not save. Try changing the name or location.")
                            return
                        }
                        saveMenu.close()
                    }
                }
            }
        }
    }

    Popup {
        id: uploadMenu

        x: parent.width / 2 - width / 2
        y: parent.height / 2 - height / 2

        modal: true
        padding: 32

        Column {
            id: uploadColumn
            width: 420
            height: 420
            spacing: 8

            Label {
                text: "Name:"
            }

            TextField {
                id: uploadNameField
                anchors {
                    left: parent.left
                    right: parent.right
                }
                placeholderText: "Name"
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

                placeholderText: "Name"
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
                        var tempFolder = StandardPaths.writableLocation(StandardPaths.TempLocation, "")
                        var stateFilename = tempFolder + "/simulation.nfy"
                        var screenshotFilename = tempFolder + "/screenshot.png"

                        neuronify.fileManager.saveState(stateFilename)
                        neuronify.saveScreenshot(screenshotFilename, function() {
                            var data = neuronify.fileManager.serializeState()
                            parse.upload("simulation.nfy", data, function(simulationFile) {
                                downloadManager.upload(
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
            PropertyChanges { target: communityBackground; state: "hidden" }
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
            PropertyChanges { target: communityBackground; state: "" }
            PropertyChanges { target: leftMenuShadow; opacity: 0.0 }
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
