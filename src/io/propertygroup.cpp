#include "propertygroup.h"

#include <QVariantMap>
#include <QMetaProperty>

PropertyGroup::PropertyGroup(QObject *parent) : QObject(parent)
{

}

QVariantMap PropertyGroup::dump()
{
    QVariantMap props;
    for(int i = 0; i < metaObject()->propertyCount(); i++) {
        QString name = metaObject()->property(i).name();
        props[name] = metaObject()->property(i).read(this);
    }
    return props;
}
