#include "abstractkernelengine.h"

AbstractKernelEngine::AbstractKernelEngine()
{
}

int AbstractKernelEngine::resolutionHeight() const
{
    return m_resolutionHeight;
}

int AbstractKernelEngine::resolutionWidth() const
{
    return m_resolutionWidth;
}

void AbstractKernelEngine::setResolutionHeight(int resolutionHeight)
{
    if (m_resolutionHeight == resolutionHeight)
        return;

    m_resolutionHeight = resolutionHeight;
    emit resolutionHeightChanged(resolutionHeight);
}

void AbstractKernelEngine::setResolutionWidth(int resolutionWidth)
{
    if (m_resolutionWidth == resolutionWidth)
        return;

    m_resolutionWidth = resolutionWidth;
    emit resolutionWidthChanged(resolutionWidth);
}
