# QmlPreviewer

Preview QML files even if they include C++ objects.
QmlPreviewer works by watching the filesystem for changes to
files listed in the .qrc resources and recompile and reload
these immediately if they change.

## Usage ##

Clone this repository into your project structure,
for instance using [git subrepo](https://github.com/ingydotnet/git-subrepo).

Include the .pri file in your .pro:

```qmake
# ... your previous code
include(qmlpreviewer/qmlpreviewer.pri)
```

Include <QmlPreviewer> in main.cpp.
Create, show and exec a QmlPreviewer object before your call app.exec()
and after registering all your C++ types.
Example:

```cpp
#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QtQml>
// --- begin QmlPreviewer ----
#include <QmlPreviewer>
// --- end QmlPreviewer -----
#include "mytype.h"

int main(int argc, char *argv[])
{
    qmlRegisterType<MyType>("MyType", 1, 0, "MyType");

    QCoreApplication::setAttribute(Qt::AA_EnableHighDpiScaling);
    QGuiApplication app(argc, argv);

    // --- begin QmlPreviewer ---
    QmlPreviewer previewer(app);
    if(previewer.show()) {
        return previewer.exec();
    }
    // --- end QmlPreviewer ---

    QQmlApplicationEngine engine;
    engine.load(QUrl(QLatin1String("qrc:/main.qml")));
    return app.exec();
}
```

Please report any issues you encounter!
