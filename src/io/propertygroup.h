#ifndef PROPERTYGROUP_H
#define PROPERTYGROUP_H

#include <QObject>
#include <QVariantMap>

class PropertyGroup : public QObject
{
    Q_OBJECT
public:
    explicit PropertyGroup(QObject *parent = 0);
    Q_INVOKABLE QVariantMap dump();

signals:

public slots:
};

#endif // PROPERTYGROUP_H
