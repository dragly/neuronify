import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import QtQuick.Dialogs 1.0
import QtQuick.Window 2.1
import QtMultimedia 5.0

import Neuronify 1.0

import "hud"
import "menus/mainmenu"
import "style"
import "io"
import "tools"

Item {
    id: root
    
    property bool muted: false
    property real volume: 1.0
    property var slots: []
    property string source
    onSourceChanged: {
        console.log(source)
    }

    Component.onCompleted: {
        for(var i = 0; i < 5; i++) {
            var soundSlot = soundSlotComponent.createObject()
            if(soundSlot) {
                slots.push(soundSlot)
            }
        }
    }
    
    function play() {
        var played = false
        for(var i in slots) {
            var slot = slots[i]
            if(!slot.playing) {
                slot.source = "qrc:/sounds/" + root.source
                slot.play()
                played = true
                break
            }
        }
        if(!played) {
            var randomSlot = parseInt(Math.random() * slots.length)
            slots[randomSlot].play()
        }
    }
    
    Component {
        id: soundSlotComponent
        SoundEffect {
            muted: root.muted
            volume: root.volume
        }
    }
}
