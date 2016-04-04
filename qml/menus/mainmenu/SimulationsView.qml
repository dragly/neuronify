import QtQuick 2.0

import "qrc:/qml/style"
import "qrc:/qml/tools"

MainMenuPage {
    id: simulationsViewRoot

    signal simulationClicked(var simulation)
    signal backClicked

    title: "New"
    clip: true

    Flickable {
        anchors.fill: parent
        anchors.margins: Style.margin
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
            Repeater {
                model: [
                    {
                        name: "Tutorial",
                        simulations: [
                            "qrc:/simulations/empty",
                            "qrc:/simulations/tutorial/tutorial_1_intro",
                            "qrc:/simulations/tutorial/tutorial_2_circuits",
                            "qrc:/simulations/tutorial/tutorial_3_creation",
                        ]
                    },
                    {
                        name: "Neurons",
                        simulations: [
                            "qrc:/simulations/neurons/passive",
                            "qrc:/simulations/neurons/inhibitory",
                            "qrc:/simulations/neurons/burst",
                            "qrc:/simulations/neurons/adaptation",
                        ]
                    },
                    {
                        name: "Inhibition",
                        simulations: [
                            "qrc:/simulations/inhibition/types_of_inhibition",
                            "qrc:/simulations/inhibition/lateral_inhibition",
                            "qrc:/simulations/inhibition/recurrent_inhibition",
                            "qrc:/simulations/inhibition/reciprocal_inhibition",
                        ]
                    },
                    {
                        name: "Mix",
                        simulations: [
                            "qrc:/simulations/mix/rythm_transformation",
                            "qrc:/simulations/mix/prolonged_activity",
                            "qrc:/simulations/mix/two_neuron_oscillator",

                        ]
                    },
                    {
                        name: "Sterratt Examples",
                        simulations: [
                            "qrc:/simulations/sterratt/fig_8_5",
                            "qrc:/simulations/sterratt/refractory_period",
                        ]
                    },
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
                                            Rectangle {
                                                color: "transparent"
                                                width: parent.width
                                                height: parent.height
                                                border.color: Style.border.color
                                                border.width: Style.border.width
                                            }
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
                                            simulationClicked(loader.item.stateSource)
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

