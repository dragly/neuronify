pragma Singleton
import QtQuick 2.0

QtObject {
    property bool debug: true
    property string serverUrl: "https://parseapi.back4app.com/"
    property string applicationId: "JffGes20AXUtdN9B6E1RkkHaS7DOxVmxJFSJgLoN"
    property string restApiKey: "bBKStu7bqeyWFTYFfM5OIes255k9XEz2Voe4fUxS"

    function setHeaders(req) {
        req.setRequestHeader("X-Parse-Application-Id", applicationId);
        req.setRequestHeader("X-Parse-REST-API-Key", restApiKey);
    }

    function processReply(req, callback) {
        if (req.readyState === XMLHttpRequest.DONE) {
            if(debug) {
                console.log(req.responseText)
            }
            if(callback) {
                var result = JSON.parse(req.responseText)
                if(result.errors !== undefined) {
                    console.log("Error parsing", req.responseText)
                    return
                }
                callback(result)
            }
        }
    }

    function serialize(obj) {
        var str = [];
        for(var p in obj) {
            str.push(encodeURIComponent(p) + "=" + encodeURIComponent(obj[p]));
        }
        return str.join("&");
    }

    function login(username, password, callback) {
        var req = new XMLHttpRequest;
        var url = serverUrl + "login"
        url += "?" + serialize({"username": username, "password": password})
        req.open("GET", url);
        setHeaders(req)
        req.onreadystatechange = function() {
            processReply(req, callback)
        }
        console.log("GET", url)
        req.send();
    }

    function get(name, callback) {
        var req = new XMLHttpRequest;
        var url = serverUrl + "classes/" + name
        req.open("GET", url);
        setHeaders(req)
        req.onreadystatechange = function() {
            processReply(req, callback)
        }
        console.log("GET", url)
        req.send();
    }

    function put(name, data, callback) {
        var req = new XMLHttpRequest;
        var url = serverUrl + "classes/" + name
        req.open("PUT", url);
        setHeaders(req)
        req.onreadystatechange = function() {
            processReply(req, callback)
        }
        console.log("PUT", url)
        req.send(data);
    }

    function post(name, data, callback) {
        var req = new XMLHttpRequest;
        var url = serverUrl + "classes/" + name
        req.open("POST", url);
        setHeaders(req)
        req.onreadystatechange = function() {
            processReply(req, callback)
        }
        console.log("POST", url)
        req.send(data);
    }
}

