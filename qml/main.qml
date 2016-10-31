import QtQuick 2.2
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.0
import Qt.labs.settings 1.0
import CuteVersioning 1.0

import "store"

ApplicationWindow {
    id: applicationWindow1

    property real startupTime: 0

//    visibility: ApplicationWindow.FullScreen
    visible: true
    width: 1136
    height: 640
    title: qsTr("Neuronify " + Version.latestTag)

    Component.onCompleted: {
        console.log("ApplicationWindow load completed " + Date.now());
        startupTime = Date.now();
    }

    Settings {
        id: settings
        property alias width: applicationWindow1.width
        property alias height: applicationWindow1.height
        property alias x: applicationWindow1.x
        property alias y: applicationWindow1.y
        property alias firstRun: neuronify.firstRun
    }

    FontLoader {
        source: "qrc:/fonts/roboto/Roboto-Regular.ttf"
    }

    FontLoader {
        source: "qrc:/fonts/roboto/Roboto-Light.ttf"
    }

    FontLoader {
        source: "qrc:/fonts/roboto/Roboto-Bold.ttf"
    }

    Neuronify {
        id: neuronify
        anchors.fill: parent
    }

    onClosing: {
        if (Qt.platform.os === "android"){
            close.accepted = false
        }
    }
}
