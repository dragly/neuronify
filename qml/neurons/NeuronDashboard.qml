import QtQuick 2.0
import QtCharts 2.1
import QtQuick.Controls 2.1
import QtQuick.Controls.Material 2.1
import QtQuick.Layouts 1.1

import Neuronify 1.0

Rectangle {
    id: root
    color: Material.background

    property real timeFactor: 1e3
    property real voltageFactor: 1e3

    width: 480
    height: 360

    TabBar {
        id: tabBar
        anchors {
            left: parent.left
            right: parent.right
            top: parent.top
        }

        TabButton {
            text: "Potentials"
        }
        TabButton {
            text: "Membrane"
        }
        TabButton {
            text: "Synapse"
        }
    }

    SwipeView {
        anchors {
            top: tabBar.bottom
            bottom: parent.bottom
            left: parent.left
            right: parent.right
        }
        currentIndex: tabBar.currentIndex
        interactive: false
        Item {
            RowLayout {
                anchors {
                    fill: parent
                    margins: 24
                }

                ChartSlider {
                    id: thresholdSlider
                    Layout.fillHeight: true
//                    Layout.preferredWidth: 64
                    chart: chart
                    series: series

                    text: "Threshold"
                    next: resetSlider.field
                    onValueChanged: {
                        if(value < resetSlider.value + 0.05) {
                            value = resetSlider.value + 0.05
                        }
                    }
                }

                ChartSlider {
                    id: resetSlider
                    Layout.fillHeight: true
//                    Layout.preferredWidth: 64
                    chart: chart
                    series: series

                    text: "Reset"
                    next: restingSlider.field
                    onValueChanged: {
                        if(value > thresholdSlider.value - 0.05) {
                            value = thresholdSlider.value - 0.05
                        }
                    }
                }

                ChartView {
                    id: chart

                    Layout.fillHeight: true
                    Layout.fillWidth: true

                    antialiasing: true
                    smooth: true
                    backgroundColor: Material.background
                    legend.visible: false
                    z: -1
                    title: "Response"

                    LineSeries {
                        id: series
                        axisX: axisX
                        axisY: axisY
                        color: Material.primary
                        width: 3
                    }

                    LineSeries {
                        id: fireSeries
                        axisX: axisX
                        axisY: axisY
                        color: Material.primary
                        width: 3
                    }

                    ValueAxis {
                        id: axisX
                        min:  0
                        max:  100
                        gridVisible: false
                        lineVisible: false
                        labelsColor: Material.foreground
                        visible: false
                    }

                    ValueAxis {
                        id: axisY
                        min: -100
                        max: 30
                        gridVisible: false
                        lineVisible: false
                        labelsColor: "grey"
//                        labelsColor: "white"
                    }
                }

                ChartSlider {
                    id: restingSlider
                    Layout.fillHeight: true
//                    Layout.preferredWidth: 64
                    chart: chart
                    series: series

                    text: "Resting"
                    next: thresholdSlider.field
                }
            }
        }
        RowLayout {
            Dial {
//                text: "Capacitance"
            }
            Dial {
//                text: "Resistance"
            }
        }
        ColumnLayout {
            Switch {
                text: "Excitatory"
            }
            Dial {

            }
            Item {
                Layout.fillWidth: true
                Layout.fillHeight: true
            }
        }
    }



    function refresh() {
        console.log("REFRESH!")
        series.clear()
        fireSeries.clear()

        var t = 0
        var dt = 1e-3
        engine.resetDynamics()
        var current = 0.1e-9
        var didFireSometime = false
        for(var i = 0; i < 200; i++) {
            if(!didFireSometime) {
                current = - (1.0 / leak.resistance) * (engine.voltage - (engine.threshold + (engine.threshold - engine.restingPotential) + 10e-3))
                engine.receiveCurrent(current, null)
            }
            engine.step(dt, true)
            //            console.log(engine.voltage * voltageFactor)

            if(engine.hasFire) {
                series.append((t - dt)*timeFactor, engine.voltage*voltageFactor)
                fireSeries.append((t - dt) * timeFactor - 1e-1, 1000e-3 * voltageFactor)
                fireSeries.append((t - dt) * timeFactor, engine.voltage * voltageFactor)
                fireSeries.append((t - dt) * timeFactor + 1e-1, 1000e-3 * voltageFactor)
                didFireSometime = true
            }
            series.append(t*timeFactor, engine.voltage*voltageFactor)

            engine.hasFire = false

            //            console.log(t, engine.voltage)
            t += dt
        }
    }

    NeuronEngine {
        id: engine

        property bool hasFire: false

        //        capacitance: capacitanceSlider.value * 1e-9
        threshold: thresholdSlider.cutValue * 1e-3
        restingPotential: restingSlider.cutValue * 1e-3
        initialPotential: resetSlider.cutValue * 1e-3

        Component.onCompleted: {
            refresh()
        }

        onRestingPotentialChanged: {
            refresh()
        }

        onInitialPotentialChanged: {
            refresh()
        }

        onThresholdChanged: {
            refresh()
        }

        onCapacitanceChanged: {
            refresh()
        }

        onFired: {
            hasFire = true
            //            console.log("FIRED")
        }


        LeakCurrent {
            id: leak

            //            resistance: resistanceSlider.value * 1e6
            onResistanceChanged: {
                refresh()
            }
        }
    }

    Column {
        //        Slider {
        //            id: capacitanceSlider
        //            from: 0.2
        //            to: 2
        //            value: 1
        //        }

        //        Slider {
        //            id: resistanceSlider
        //            from: 2
        //            to: 50
        //            value: 25
        //        }

        //        Slider {
        //            id: restingSlider
        //            from: -100
        //            to: 30
        //            value: -70
        //        }
    }
}
