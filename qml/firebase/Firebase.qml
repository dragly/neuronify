pragma Singleton
import QtQuick 2.0
import QtQuick.Controls 2.0
import QtQuick.Window 2.0

QtObject {
    property string serverUrl: "https://neuronify.firebaseio.com/"
    property string auth

    function buildUrl(name) {
        if(auth) {
            return serverUrl + name + ".json?auth=" + auth
        }
        return serverUrl + name + ".json"
    }

    function get(name, callback) {
        console.log("Get me?")
        var url = buildUrl(name)
        var req = new XMLHttpRequest()
        req.open("GET", url)
        req.onreadystatechange = function() {
            if(req.readyState !== XMLHttpRequest.DONE) {
                return
            }
            if(req.status !== 200) {
                console.log("Error", req.status, req.statusText)
                console.log(req.responseText)
                return
            }
            callback(req)
        }
        //    req.setRequestHeader("Authorization", authorization)
        req.send()
    }

    function patch(name, data, callback) {
        var url = buildUrl(name)
        var req = new XMLHttpRequest()
        req.onreadystatechange = function() {
            if(req.readyState != XMLHttpRequest.DONE) {
                return
            }
            if(req.status != 200) {
                console.log("ERROR:", req.status, req.statusText)
                console.log(req.responseText)
                return
            }
            callback(req)
        }
        req.open("POST", url)
        req.setRequestHeader("X-HTTP-Method-Override", "PATCH")
        //    req.setRequestHeader("Authorization", Firebase.authorization)
        req.send(JSON.stringify(data))
    }

    function put(name, data, callback) {
        var url = buildUrl(name)
        var req = new XMLHttpRequest()
        req.onreadystatechange = function() {
            if(req.readyState != XMLHttpRequest.DONE) {
                return
            }
            if(req.status != 200) {
                console.log("ERROR:", req.status, req.statusText)
                console.log(req.responseText)
                return
            }
            callback(req)
        }
        req.open("PUT", url)
        //    req.setRequestHeader("Authorization", Firebase.authorization)
        req.send(JSON.stringify(data))
    }

    function remove(name, callback) {
        var url = buildUrl(name)
        var req = new XMLHttpRequest()
        req.onreadystatechange = function() {
            if(req.readyState != XMLHttpRequest.DONE) {
                return
            }
            if(req.status != 200) {
                console.log("ERROR:", req.status, req.statusText)
                console.log(req.responseText)
                return
            }
            callback(req)
        }
        req.open("DELETE", url)
        req.send()
    }

    function post(name, data, callback) {
        var url = buildUrl(name)
        var req = new XMLHttpRequest()
        req.onreadystatechange = function() {
            if(req.readyState != XMLHttpRequest.DONE) {
                return
            }
            if(req.status != 200) {
                console.log("ERROR:", req.status, req.statusText)
                console.log(req.responseText)
                return
            }
            console.log("Post result:", req.status, req.responseText)
            callback(req)
        }
        req.open("POST", url)
        req.send(JSON.stringify(data))
    }
}
