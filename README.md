# Neuronify

Visual neuron simulator using integrate-and-fire neurons.

## Building Neuronify

### Prerequisites

1. Download Qt from http://www.qt.io/download-open-source/
2. Run the installer and follow the instructions

### Downloading and building

3. Clone the git repo: git@github.com:CINPLA/neuronify.git
4. Open Qt Creator
5. In Qt Creator, use Open Project, go to the neuronify folder and open nestify.pro or neuronify.pro, whichever is present
6. Under configure project that shows up, mark Desktop Qt 5.4.0 GCC and unmark Desktop before pressing Configure Project
7. Press Run (green triangle) and it should hopefully build and run.

### Creating a deployable .zip on Windows

1. In Qt Creator, you may choose to build using the MinGW or Visual Studio 2013 (MSVC) compilers depending on your installation. We recommend Visual Studio because this adds support for systems without OpenGL drivers already installed.
2. Compile in release mode. Otherwise the build will be gigantic.
3. Open Windows PowerShell.
4. Run the following command:
    - if you are using MinGW:

        $env:Path = "C:\Qt\5.4\mingw491_32\bin"

    - if you are using MSVC:

        $env:Path = "C:\Qt\5.4\msvc2013\bin"

5. Note that the names of the folders may vary if you have a different Qt version. These above are the folders for Qt 5.4 and Visual Studio 2013.
5. Use cd to navigate to the build folder, above the release and debug folders.
5. Delete everything except neuronify.exe inside the release folder.
6. Run the following command to let Qt help you with deployment:

    windeployqt.exe -quick -quickwidgets -multimediaquick -multimedia -opengl -quickparticles -quickwidgets -qmldir ../neuronify release

7. Next, you will need to copy the following files to your build folder manually:
    - if you are using MinGW:

        C:\Windows\SysWOW64\msvcp120.dll
        C:\Windows\SysWOW64\msvcr120.dll
        C:\Qt\5.4\mingw491_32\bin\libgcc_s_dw2-1.dll
        C:\Qt\5.4\mingw491_32\bin\libstdc++-6.dll
        C:\Qt\5.4\mingw491_32\bin\libwinpthread-1.dll

    - if you are using MSVC:

        C:\Windows\SysWOW64\msvcp120.dll
        C:\Windows\SysWOW64\msvcr120.dll

8. Zip the whole folder, and you should be good to go
9. Note that these are the files you need to copy if you are using Qt 5.4 and Visual Studio 2013. If you are using a newer version, you may want to use a newer set of files. Please refer to the documentation for more information:

http://doc.qt.io/qt-5/windows-deployment.html

### Creating a deployable .dmg on Mac

Simply run the following command in the build-directory:

    /path/to/Qt/installation/clang_64/bin/macdeployqt neuronify.app -qmldir=../neuronify -dmg

This should create a .dmg that can be used on machines without Qt installed.

### Building the documentation ###

You should first install the index files for Qt:

    sudo apt-get install qt5-doc qt5-doc-html qt5-default qt5-qmake qtquickcontrols5-doc-html qtlocation5-doc

Afterwards, run ./make-docs.sh in the doc folder to generate the documentation.
This will be generated in a folder named neuronify-docs above the source tree.
