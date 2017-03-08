import QtQuick 2.0

Backend {
    id: root
    property bool debug: false
    property string serverUrl
    property string applicationId
    property string restApiKey
    property string sessionToken
    property string objectId
    readonly property bool loggedIn: {
        console.log("Checking", sessionToken, objectId)
        return sessionToken !== "" && objectId !== ""
    }

    onLoggedInChanged: {
        console.log("Logged in", loggedIn)
    }

    onSessionTokenChanged: {
        console.log("Session token changed", sessionToken)
        if(!sessionToken) {
            objectId = ""
            return
        }
        var req = new XMLHttpRequest;
        var url = serverUrl + "users/me"
        req.open("GET", url);
        setHeaders(req)
        req.onreadystatechange = function() {
            var result = processReply(req, function(result) {
                objectId = result.objectId
                console.log("Successfully logged in with previous token.")
            })
        }
        console.log("GET", url)
        req.send();
    }

    function setHeaders(req) {
        req.setRequestHeader("X-Parse-Application-Id", applicationId);
        req.setRequestHeader("X-Parse-REST-API-Key", restApiKey);
        if(sessionToken) {
            console.log("Using session token", sessionToken)
            req.setRequestHeader("X-Parse-Session-Token", sessionToken);
        }
    }

    function processReply(req, callback) {
        if (req.readyState === XMLHttpRequest.DONE) {
            if(debug) {
                console.log("Response:", req.responseText)
            }
            if(callback) {
                var result = JSON.parse(req.responseText)
                if(result.errors !== undefined) {
                    console.log("Error parsing", req.responseText)
                    return
                }
                callback(result)
                return result
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
            var result = processReply(req, function(result) {
                sessionToken = result.sessionToken
                if(callback) {
                    callback(result)
                }
            })
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
        req.send(JSON.stringify(data));
    }

    function post(name, data, callback) {
        var req = new XMLHttpRequest;
        var url = serverUrl + "classes/" + name
        req.open("POST", url);
        setHeaders(req)
        req.onreadystatechange = function() {
            processReply(req, callback)
        }
        console.log("POST", url, data)
        console.log("Data", JSON.stringify(data))
        req.send(JSON.stringify(data));
    }

    function upload(name, data, callback) {
        var req = new XMLHttpRequest;
        var url = serverUrl + "files/" + name
        req.open("POST", url);
        setHeaders(req)
        req.setRequestHeader("Content-Type", "text/plain");
        req.onreadystatechange = function() {
            if(req.readyState == XMLHttpRequest.DONE) {
                var result = JSON.parse(req.responseText)
                callback(result)
            }
        }
        console.log("POST", url)
        req.send(JSON.stringify(data, null, 4));
    }

    function uploadBinary(name, data, callback) {

//        var req = new XMLHttpRequest;
//        var url = serverUrl + "files/" + name
//        req.open("POST", url);
//        setHeaders(req)
//        req.setRequestHeader("Content-Type", "text/plain");
//        req.onreadystatechange = function() {
//            if(req.readyState == XMLHttpRequest.DONE) {
//                var result = JSON.parse(req.responseText)
//                callback(result)
//            }
//        }
//        console.log("POST", url)
//        req.send(JSON.stringify(data, null, 4));
    }

    function logout() {
        sessionToken = ""
        if(loggedIn) {
            console.error("ERROR: Could not log out user:", sessionToken, objectId)
        }
    }
}

