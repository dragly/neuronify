#ifndef DOGKERNELENGINE_H
#define DOGKERNELENGINE_H

#include "abstractkernelengine.h"

class DogKernelEngine : public AbstractKernelEngine
{
    Q_OBJECT
    Q_PROPERTY(double centerWeight READ centerWeight
               WRITE setCenterWeight NOTIFY centerWeightChanged)
    Q_PROPERTY(double centerExp READ centerExp
               WRITE setCenterExp NOTIFY centerExpChanged)
    Q_PROPERTY(double surroundWeight READ surroundWeight
               WRITE setSurroundWeight NOTIFY surroundWeightChanged)
    Q_PROPERTY(double surroundExp READ surroundExp
               WRITE setSurroundExp NOTIFY surroundExpChanged)
    Q_PROPERTY(bool isOffCenter READ isOffCenter WRITE setIsOffCenter NOTIFY isOffCenterChanged)

public:
    DogKernelEngine();

    double centerWeight() const;
    double centerExp() const;
    double surroundWeight() const;
    double surroundExp() const;

    // AbstractKernelEngine interface
public:
    virtual void createKernel(vector<vector<double> > *spatial);

    bool isOffCenter() const
;

public slots:
    void setCenterWeight(double centerWeight);
    void setCenterExp(double centerExp);
    void setSurroundWeight(double surroundWeight);
    void setSurroundExp(double surroundExp);
    void setIsOffCenter(bool isOffCenter);

signals:
    void centerWeightChanged(double centerWeight);
    void centerExpChanged(double centerExp);
    void surroundWeightChanged(double surroundWeight);
    void surroundExpChanged(double surroundExp);

    void isOffCenterChanged(bool isOffCenter);

private:
    double advance(double x, double y);
    double m_centerWeight;
    double m_centerExp;
    double m_surroundWeight;
    double m_surroundExp;
    bool m_isOffCenter= false;
};

#endif // DOGKERNELENGINE_H
