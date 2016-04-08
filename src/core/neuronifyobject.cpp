#include "neuronifyobject.h"

NeuronifyObject::NeuronifyObject(QQuickItem *parent) : QQuickItem(parent)
{

}

void NeuronifyObject::addSavedPropertyGroup(PropertyGroup *propertyGroup)
{
    m_savedProperties.append(propertyGroup);
}
