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

    Component.onCompleted: {
        for(var i = 0; i < 5; i++) {
            var soundSlot = soundSlotComponent.createObject()
            if(soundSlot) {
                slots.push(soundSlot)
            }
        }
    }
    
    function play() {
        if(muted) {
            return
        }
        
        for(var i in slots) {
            var slot = slots[i]
            if(!slot.playing) {
                slot.volume = volume
                slot.play()
                break
            }
        }
    }
    
    Component {
        id: soundSlotComponent
        SoundEffect {
            source: "qrc:/sounds/thump.wav"
        }
    }
}
