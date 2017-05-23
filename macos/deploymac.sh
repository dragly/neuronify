#!/bin/bash
# http://thegreyblog.blogspot.no/2014/06/os-x-creating-packages-from-command_2.html
# http://blog.qt.io/blog/2012/04/03/how-to-publish-qt-applications-in-the-mac-app-store-2/
# http://wiki.phisys.com/index.php/How-To_Qt5.3_Mac_AppStore#Entitlements
# Create icons with
# iconutil -c icns icon.iconset
# sudo installer -store -pkg Atomify.pkg -target /
mkdir tmp
rm -r tmp/*
cp -r Neuronify.app tmp
cd tmp
xattr -cr Neuronify.app
~/Qt/5.8/clang_64/bin/macdeployqt Neuronify.app -qmldir=/repos/neuronify/qml -codesign="3rd Party Mac Developer Application: Anders Hafreager" -appstore-compliant
cd "Neuronify.app"
find . -name *.dSYM | xargs -I $ rm -R $
cd ..
cp ../Neuronify.app/Contents/MacOS/neuronify Neuronify.app/Contents/MacOS/
codesign -s "3rd Party Mac Developer Application: Anders Hafreager" --entitlements /repos/neuronify/macos/entitlements.plist Neuronify.app
productbuild --component Neuronify.app /Applications --sign "3rd Party Mac Developer Installer: Anders Hafreager" Neuronify.pkg
# Do this again haha
~/Qt/5.8/clang_64/bin/macdeployqt Neuronify.app -qmldir=/repos/neuronify/qml -codesign="3rd Party Mac Developer Application: Anders Hafreager" -appstore-compliant
productbuild --component Neuronify.app /Applications --sign "3rd Party Mac Developer Installer: Anders Hafreager" Neuronify.pkg
cd ..