import QtQuick 2.0

import "qrc:/qml/style"
import "qrc:/qml/tools"

Item {
    id: simulationsViewRoot

    signal simulationClicked(var simulation)
    signal backClicked
    clip: true

    Flickable {
        anchors.fill: parent
        anchors.margins: Style.baseMargin
        contentWidth: width
        contentHeight: categories.height
        flickableDirection: Flickable.VerticalFlick
        Column {
            id: categories
            anchors{
                left: parent.left
                right: parent.right
            }
            spacing: Style.baseMargin
            Item {
                width: 100
                height: Style.touchableSize
            }
            Repeater {
                model: [
                    {
                        name: "Simple",
                        simulations: [
                            "qrc:/simulations/singleCell",
                        ]
                    },
                    {
                        name: "Inhibition",
                        simulations: [
                            "qrc:/simulations/lateralInhibition",
                            "qrc:/simulations/recurrentInhibition",
                            "qrc:/simulations/twoNeuronOscillator",
                        ]
                    },
                    {
                        name: "Visual system",
                        simulations: [
                            "qrc:/simulations/light",
                        ]
                    },
                    {
                        name: "Sterratt Examples",
                        simulations: [
                            "qrc:/simulations/sterratt/fig_8_5",
                        ]
                    }
                ]
                Column {
                    spacing: Style.baseMargin
                    Text {
                        text: modelData.name
                        font: Style.button.font
                        color: Style.button.color
                    }
                    Flickable {
                        width: categories.width
                        height: simulationsRow.height
                        contentHeight: height
                        contentWidth: simulationsRow.width
                        flickableDirection: Flickable.HorizontalFlick
                        Row {
                            id: simulationsRow
                            spacing: Style.baseMargin * 0.5
                            Repeater {
                                model: modelData.simulations
                                Item {
                                    width: simulationColumn.width
                                    height: simulationColumn.height
                                    SimulationLoader {
                                        id: loader
                                        folder: modelData
                                    }
                                    Column {
                                        id: simulationColumn
                                        Image {
                                            id: simulationImage
                                            property int simulationsCount: simulationsViewRoot.width / (Style.touchableSize * 2.5 + Style.baseMargin)

                                            width: (simulationsViewRoot.width - Style.baseMargin) / (simulationsCount + 0.3) - simulationsRow.spacing
                                            height: width

                                            source: loader.item ? loader.item.screenshotSource : ""
                                            smooth: true
                                            antialiasing: true
                                            fillMode: Image.PreserveAspectCrop
                                        }
                                        Text {
                                            id: title
                                            font: Style.text.font
                                            color: Style.text.color
                                            text: loader.item ? loader.item.name : "N/A"
                                            width: simulationImage.width
                                            wrapMode: Text.WrapAtWordBoundaryOrAnywhere
                                            horizontalAlignment: Text.AlignHCenter

                                        }
                                    }
                                    MouseArea {
                                        anchors.fill: parent
                                        onClicked: {
                                            simulationClicked(loader.item)
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
}

