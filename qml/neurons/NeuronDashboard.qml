import QtQuick 2.0
import QtCharts 2.1
import QtQuick.Controls 2.1
import QtQuick.Controls.Material 2.1
import QtQuick.Layouts 1.1

import Neuronify 1.0

import "qrc:/qml/controls"
import "qrc:/qml/neurons"

Rectangle {
    id: root
    color: Material.background

    property real timeFactor: 1e3
    property real voltageFactor: 1e3
    property Neuron neuron
    property NeuronEngine neuronEngine: NeuronEngine {}
    property LeakCurrent leakCurrent: LeakCurrent {}
    property bool completed: false

    Component.onCompleted: {
//        completed = true
    }

    width: 480
    height: 360

    MouseArea {
        anchors.fill: parent
    }

    TabBar {
        id: tabBar
        anchors {
            left: parent.left
            right: parent.right
            top: parent.top
        }

        currentIndex: 0

        TabButton {
            text: "General"
        }
        TabButton {
            text: "Potentials"
        }
        TabButton {
            text: "Membrane"
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
            GridLayout {
                anchors {
                    fill: parent
                    margins: 24
                }

                columns: 2

                Label {
                    text: "Label:"
                    Layout.fillWidth: true
                }
                TextField {
                    id: labelField
                    Layout.fillWidth: true
                    text: neuron.label
                    selectByMouse: true
                    Binding {
                        target: neuron
                        property: "label"
                        value: labelField.text
                    }
                    Binding {
                        target: labelField
                        property: "text"
                        value: neuron.label
                    }
                }

                Label {
                    text: "Excitatory"
                    Layout.fillWidth: true
                    MouseArea {
                        anchors.fill: parent
                        onClicked: {
                            excitatorySwitch.checked = !excitatorySwitch.checked
                        }
                    }
                }
                Switch {
                    id: excitatorySwitch
                    Layout.alignment: Qt.AlignRight
                    Binding {
                        target: neuron
                        property: "inhibitory"
                        value: !excitatorySwitch.checked
                    }
                    Binding {
                        target: excitatorySwitch
                        property: "checked"
                        value: !neuron.inhibitory
                    }
                }

                BoundSlider {
                    Layout.columnSpan: 2
                    Layout.fillWidth: true

                    target: neuronEngine
                    property: "refractoryPeriod"
                    text: "Refractory period"
                    unit: "ms"
                    minimumValue: 0.0e-3
                    maximumValue: 50e-3
                    unitScale: 1e-3
                    stepSize: 1e-3
                    precision: 1
                }

                Item {
                    Layout.fillHeight: true
                    Layout.fillWidth: true
                }
            }
        }

        Item {
            NeuronEngine {
                id: testEngine

                property bool hasFire: false

                function refresh() {
                    series.clear()
                    fireSeries.clear()

                    var t = 0
                    var dt = 1e-3
                    testEngine.resetDynamics()
                    var current = 0.1e-9
                    var didFireSometime = false
                    for(var i = 0; i < 200; i++) {
                        if(!didFireSometime) {
                            current = - (1.0 / leak.resistance) * (testEngine.voltage - (testEngine.threshold + (testEngine.threshold - testEngine.restingPotential) + 10e-3))
                            testEngine.receiveCurrent(current, null)
                        }
                        testEngine.step(dt, true)
                        //            console.log(engine.voltage * voltageFactor)

                        if(testEngine.hasFire) {
                            series.append((t - dt)*timeFactor, testEngine.voltage*voltageFactor)
                            fireSeries.append((t - dt) * timeFactor - 1e-1, 1000e-3 * voltageFactor)
                            fireSeries.append((t - dt) * timeFactor, testEngine.voltage * voltageFactor)
                            fireSeries.append((t - dt) * timeFactor + 1e-1, 1000e-3 * voltageFactor)
                            didFireSometime = true
                        }
                        series.append(t*timeFactor, testEngine.voltage*voltageFactor)

                        testEngine.hasFire = false

                        //            console.log(t, engine.voltage)
                        t += dt
                    }
                }

                //        capacitance: capacitanceSlider.value * 1e-9
                threshold: thresholdSlider.value * 1e-3
                restingPotential: restingSlider.value * 1e-3
                initialPotential: resetSlider.value * 1e-3

                Component.onCompleted: {
                    testEngine.refresh()
                }

                onRestingPotentialChanged: {
                    testEngine.refresh()
                }

                onInitialPotentialChanged: {
                    testEngine.refresh()
                }

                onThresholdChanged: {
                    testEngine.refresh()
                }

                onCapacitanceChanged: {
                    testEngine.refresh()
                }

                onFired: {
                    hasFire = true
                    //            console.log("FIRED")
                }


                LeakCurrent {
                    id: leak

                    //            resistance: resistanceSlider.value * 1e6
                    onResistanceChanged: {
                        testEngine.refresh()
                    }
                }
            }
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
                    Binding {
                        target: neuronEngine
                        property: "threshold"
                        value: thresholdSlider.value / voltageFactor
                        when: thresholdSlider.ready
                    }
                    Binding {
                        target: thresholdSlider
                        property: "value"
                        value: neuronEngine.threshold * voltageFactor
                        when: thresholdSlider.ready
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
//                    value: neuronEngine.initialPotential
                    Binding {
                        target: neuronEngine
                        property: "initialPotential"
                        value: resetSlider.value / voltageFactor
                        when: resetSlider.ready
                    }
                    Binding {
                        target: resetSlider
                        property: "value"
                        value: neuronEngine.initialPotential * voltageFactor
                        when: resetSlider.ready
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

                    MouseArea {
                        anchors.fill: parent
                        onClicked: focus = true
                    }

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
                    Binding {
                        target: neuronEngine
                        property: "restingPotential"
                        value: restingSlider.value / voltageFactor
                        when: restingSlider.ready
                    }
                    Binding {
                        target: restingSlider
                        property: "value"
                        value: neuronEngine.restingPotential * voltageFactor
                        when: restingSlider.ready
                    }
                }
            }
        }

        Item {
            RowLayout {
                anchors {
                    top: parent.top
                    left: parent.left
                    right: parent.right
                    bottom: timeConstant.top
                    margins: 24
                }

                Item {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                }

                NumberSlider {
                    Layout.fillHeight: true
                    property: "resistance"
                    target: leakCurrent
                    name: "Resistance"
                    factor: 1e6
                    from: 1e6
                    to: 200e6
                    precision: 0
                    unit: "MΩ"
                }

                Image {
                    Layout.fillHeight: true
                    Layout.fillWidth: true
                    Layout.maximumWidth: height
                    smooth: true
                    antialiasing: true
                    fillMode: Image.PreserveAspectFit
                    source: "qrc:/images/dashboard/integrate-and-fire.png"
                }

                NumberSlider {
                    Layout.fillHeight: true

                    target: neuronEngine
                    property: "capacitance"
                    name: "Capacitance"
                    from: 0.1e-9
                    to: 2e-9
                    factor: 1e-9
                    precision: 2
                    unit: "nF"
                }

                Item {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                }
            }
            Label {
                id: timeConstant
                anchors {
                    bottom: parent.bottom
                    horizontalCenter: parent.horizontalCenter
                    margins: 24
                }
                text: "Time constant: " + (leakCurrent.resistance * neuronEngine.capacitance * 1e3).toFixed(1) + " ms"
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
