import QtQuick 2.5
import QtQuick.Controls 2.1
import QtQuick.Dialogs 1.0
import QtQuick.Layouts 1.1
import QtQuick.Particles 2.0
import QtQuick.Window 2.1

import QtCharts 2.1
import QtMultimedia 5.5
import Qt.labs.settings 1.0

import Neuronify 1.0
import CuteVersioning 1.0
import QtGraphicalEffects 1.0

import "qrc:/qml/hud"
import "qrc:/qml/menus/mainmenu"
import "qrc:/qml/style"
import "qrc:/qml/io"
import "qrc:/qml/tools"
import "qrc:/qml/controls"

Item {
    Neuronify {
        id: neuronify
        anchors.fill: parent
        onSimulationLoaded: {
            playbackControls.revealTemporarily()
        }
    }

    ButtonColumn {
        ButtonColumnButton {
            source: "qrc:/images/tools/mainmenu.png"
            onClicked: {
                mainMenu.revealed = true
            }
        }
        ButtonColumnButton {
            source: "qrc:/images/tools/create.png"
            onClicked: {
                creationMenu.revealed = !creationMenu.revealed
            }
        }
        ButtonColumnButton {
            source: "qrc:/images/tools/playback.png"
            onClicked: {
                playbackControls.toggleRevealPermanently()
            }
        }
        ButtonColumnButton {
            source: "qrc:/images/tools/properties.png"
            onClicked: {
                propertiesPanel.open()
            }
        }
    }

    DeleteButton {
        anchors {
            right: parent.right
            bottom: parent.bottom
        }
        revealed: neuronify.activeObject ? true : false
        onClicked: {
            deleteSelected()
        }
    }

    PlaybackControls {
        id: playbackControls
        revealed: true
        autoHideEnabled: neuronify.running
    }

    CreationMenu {
        id: creationMenu

        onRevealedChanged: {
            if(revealed) {
                neuronify.focus = false
            } else {
                neuronify.focus = true
            }
        }

        onDroppedEntity: {
            var workspacePosition = controlParent.mapToItem(neuronLayer, properties.x, properties.y)
            properties.x = workspacePosition.x
            properties.y = workspacePosition.y
            neuronify.createEntity(fileUrl, properties)
        }

        onDeleteEverything: {
            neuronify.deleteEverything()
        }

    }

    PropertiesPanel {
        id: propertiesPanel

        advanced: neuronify.advanced
        snappingEnabled: neuronify.snappingEnabled
        activeObject: neuronify.activeObject
        workspace: neuronify.workspace

        onRevealedChanged: {
            if(revealed) {
                neuronify.clickMode = "selection"
            } else {
                neuronify.focus = true
            }
        }

        onResetDynamics: {
            for(var i in graphEngine.nodes) {
                var entity = graphEngine.nodes[i];
                if(entity.engine) {
                    entity.engine.resetDynamics();
                }
            }
            for(var i in graphEngine.edges) {
                var edge = graphEngine.edges[i];
                if(edge.engine) {
                    edge.engine.resetDynamics();
                }
            }
        }

        onResetProperties: {
            for(var i in graphEngine.nodes) {
                var entity = graphEngine.nodes[i];
                if(entity.engine) {
                    entity.engine.resetProperties();
                }
            }
            for(var i in graphEngine.edges) {
                var edge = graphEngine.edges[i];
                if(edge.engine) {
                    edge.engine.resetProperties();
                }
            }
        }

        onSaveToOpened: {
            propertiesPanel.revealed = false
            saveTimer.start(1000)
        }

        Timer {
            id: saveTimer
            onTriggered: {
                saveState(StandardPaths.originalSimulationLocation(currentSimulationUrl));
                var imageUrl = StandardPaths.toLocalFile(StandardPaths.originalSimulationLocation(currentSimulationUrl)).replace(".nfy", ".png")

                workspaceFlickable.grabToImage(function(result) {
                    result.saveToFile(imageUrl);
                }, Qt.size(workspaceFlickable.width / 3.0, workspaceFlickable.height / 3.0));
            }
        }

        Binding {
            target: neuronify
            property: "advanced"
            value: propertiesPanel.advanced
        }

        Binding {
            target: propertiesPanel
            property: "advanced"
            value: neuronify.advanced
        }

        Binding {
            target: neuronify
            property: "snappingEnabled"
            value: propertiesPanel.snappingEnabled
        }

        Binding {
            target: propertiesPanel
            property: "snappingEnabled"
            value: neuronify.snappingEnabled
        }
    }

    ConnectionMenu {
        visible: neuronify.clickMode === "connectMultipleToThis" || neuronify.clickMode === "connectMultipleFromThis"
        fromThis: neuronify.clickMode === "connectMultipleFromThis"
        onDoneClicked: {
            neuronify.clickMode = "selection"
        }
    }

    MainMenu {
        id: mainMenu

        property bool wasRunning: true

        focus: true

        anchors.fill: parent

        onRevealedChanged: {
            if(revealed) {
                wasRunning = neuronify.running
                neuronify.focus = false
            } else {
                neuronify.focus = true
            }
        }

        onContinueClicked: {
            mainMenu.revealed = false
        }

        onNewClicked: {
            neuronify.loadSimulation("qrc:/simulations/empty/empty.nfy")
            mainMenu.revealed = false
        }

        onLoadSimulation: {
            neuronify.loadSimulation(simulation)
            mainMenu.revealed = false
            //neuronify.running = wasRunning
        }

        onSaveSimulation: {
            neuronify.save(simulation)
            mainMenu.revealed = false
        }

        onSaveSimulationRequested: {
            fileManager.showSaveDialog()
        }

        onLoadSimulationRequested: {
            fileManager.showLoadDialog()
        }

        onRequestScreenshot: {
            var aspectRatio = workspaceFlickable.width / workspaceFlickable.height;
            var imageWidth;
            if(workspaceFlickable.width > workspaceFlickable.height) {
                imageWidth = workspaceFlickable.width / 3.0; // three icons per row in save view
            } else {
                imageWidth = workspaceFlickable.width / 2.0; // two icons per row in save view
            }
            workspaceFlickable.grabToImage(callback, Qt.size(imageWidth, imageWidth / aspectRatio));
        }
    }
}
