import QtQuick 2.2
import QtQuick.Controls 2.1
import Qt.labs.settings 1.0
import CuteVersioning 1.0
import QtQuick.Window 2.0

import "qrc:/qml/style"

ApplicationWindow {
    id: root

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

    onWidthChanged: {
        resetStyle()
    }

    onHeightChanged: {
        resetStyle()
    }

    onClosing: {
        // Hack to keep back button from closing app
        console.log("onClosing")
        if (Qt.platform.os === "android"){
            close.accepted = false
            return
        }
        if(!mainDesktop.tryClose()) {
            close.accepted = false
            return
        }
        console.log("Neuronify closing...")
    }

    function resetStyle() {
        Style.reset(width, height, Screen.pixelDensity)
    }

    Settings {
        id: settings
        property alias width: root.width
        property alias height: root.height
        property alias x: root.x
        property alias y: root.y
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

    FontLoader {
        source: "https://github.com/google/material-design-icons/raw/master/iconfont/MaterialIcons-Regular.ttf"
    }

    MainDesktop {
        id: mainDesktop
        anchors.fill: parent
        focus: true
        onRequestClose: {
            console.log("Close requested")
            root.close()
        }
    }
}
