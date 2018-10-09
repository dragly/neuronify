#!/bin/bash
# http://thegreyblog.blogspot.no/2014/06/os-x-creating-packages-from-command_2.html
# http://blog.qt.io/blog/2012/04/03/how-to-publish-qt-applications-in-the-mac-app-store-2/
# http://wiki.phisys.com/index.php/How-To_Qt5.3_Mac_AppStore#Entitlements
# Create icons with
# iconutil -c icns icon.iconset
# Create dmg from terminal
# http://www.theinstructional.com/guides/disk-management-from-the-command-line-part-3

# If qmake is not in path, specify it here
export PATH=$PATH:/Users/anderhaf/Qt/5.10.1/clang_64/bin/

# Verify that 10.9 SDK exists
if [ ! -d "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX10.9.sdk" ]; then
  echo "Could not find 10.9 SDK at /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX10.9.sdk"
  echo "Please download SDK from https://github.com/phracker/MacOSX-SDKs"
  exit 1
fi

mkdir build-pkg
cd build-pkg
qmake ../../
# Force using 10.9 SDK even if current OS is newer. Not sure how to fix this in qmake call
sed -i "" 's/MacOSX10.10.sdk/MacOSX10.9.sdk/g' Makefile
sed -i "" 's/MacOSX10.11.sdk/MacOSX10.9.sdk/g' Makefile
sed -i "" 's/MacOSX10.12.sdk/MacOSX10.9.sdk/g' Makefile
sed -i "" 's/MacOSX10.13.sdk/MacOSX10.9.sdk/g' Makefile
make -j8

# First create folder to create dmg
mkdir Neuronify
cp -r Neuronify.app Neuronify
cd Neuronify
ln -s /Applications Applications
xattr -cr Neuronify.app
macdeployqt Neuronify.app -qmldir=../../../qml -codesign="Developer ID Application: Anders Hafreager (4XKET6P69R)"
cd ..

# Create dmg with shortcut to Applications
mkdir dmg
cd dmg
hdiutil create Neuronify.dmg -volname "Neuronify" -srcfolder ../Neuronify/
cp Neuronify.dmg ../../Neuronify-v1.3.2-macos.dmg
cd ..

# Now create folder for pkg
mkdir pkg
rm -rf pkg/*
cp -r Neuronify.app pkg
cd pkg
xattr -cr Neuronify.app
macdeployqt Neuronify.app -dmg -qmldir=../../../qml -codesign="Developer ID Application: Anders Hafreager (4XKET6P69R)"
cd "Neuronify.app"
find . -name *.dSYM | xargs -I $ rm -R $
cd ..
productbuild --component Neuronify.app /Applications --sign "Developer ID Installer: Anders Hafreager" Neuronify-1.3.2-macos-installer.pkg
cp Neuronify-v1.3.2-macos-installer.pkg ../../
