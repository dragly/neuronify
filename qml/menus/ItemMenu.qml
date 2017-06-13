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
import "qrc:/qml/hud"
import "qrc:/qml/menus/filemenu"
import "qrc:/qml/menus/mainmenu"
import "qrc:/qml/tools"
import "qrc:/qml/store"
import "qrc:/qml/style"
import "qrc:/qml/ui"

Item {
    id: itemMenu
    
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
                top: parent.top
                topMargin: 16
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
                        font.pixelSize: 14
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
                        property real itemHeight: (height - spacing * (rows - 1)) / rows - 1
                        property real itemWidth: (width - spacing * (columns - 1)) / columns - 1
                        
                        anchors {
                            left: parent.left
                            right: parent.right
                            leftMargin: 18
                            rightMargin: 18
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
