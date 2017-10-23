#include "abstractkernelengine.h"

AbstractKernelEngine::AbstractKernelEngine()
{
        updateX();
        updateY();
        m_shift_x = 0.0;
        m_shift_y = 0.0;
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
    updateY();
    emit resolutionHeightChanged(resolutionHeight);
    emit needsRecreation();
}

void AbstractKernelEngine::setResolutionWidth(int resolutionWidth)
{
    if (m_resolutionWidth == resolutionWidth)
        return;

    m_resolutionWidth = resolutionWidth;
    updateX();
    emit resolutionWidthChanged(resolutionWidth);
    emit needsRecreation();
}

void AbstractKernelEngine::setShift_x(double shift_x)
{
    qWarning("Floating point comparison needs context sanity check");
    if (qFuzzyCompare(m_shift_x, shift_x))
        return;

    m_shift_x = shift_x;
    emit shift_xChanged(m_shift_x);
    emit needsRecreation();
}

void AbstractKernelEngine::setShift_y(double shift_y)
{
    qWarning("Floating point comparison needs context sanity check");
    if (qFuzzyCompare(m_shift_y, shift_y))
        return;

    m_shift_y = shift_y;
    emit shift_yChanged(m_shift_y);
    emit needsRecreation();

}

vector<double> AbstractKernelEngine::y() const
{
    return m_y;
}

double AbstractKernelEngine::shift_x() const
{
    return m_shift_x;
}

double AbstractKernelEngine::shift_y() const
{
    return m_shift_y;
}

vector<double> AbstractKernelEngine::x() const
{
    return m_x;
}

void AbstractKernelEngine::updateY()
{
    double step = 1./(m_resolutionHeight-1);
    m_y.resize(m_resolutionHeight);
    for(int i=0; i < m_resolutionHeight; i++){
        m_y[i] = -0.5 + step * i;
    }
}

void AbstractKernelEngine::updateX()
{
    double step = 1./(m_resolutionWidth-1);

    m_x.resize(m_resolutionWidth);
    for(int i=0; i < m_resolutionWidth; i++){
        m_x[i] = -0.5 + step * i;
    }
}

