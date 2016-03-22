#ifndef GABORKERNELENGINE_H
#define GABORKERNELENGINE_H

#include "abstractkernelengine.h"

class GaborKernelEngine : public AbstractKernelEngine
{
    Q_OBJECT
    Q_PROPERTY(double sigmaX READ sigmaX WRITE setSigmaX NOTIFY sigmaXChanged)
    Q_PROPERTY(double sigmaY READ sigmaY WRITE setSigmaY NOTIFY sigmaYChanged)
    Q_PROPERTY(double k READ k WRITE setK NOTIFY kChanged)
    Q_PROPERTY(double phi READ phi WRITE setPhi NOTIFY phiChanged)
    Q_PROPERTY(double theta READ theta WRITE setTheta NOTIFY thetaChanged)

public:
    GaborKernelEngine();

    // AbstractKernelEngine interface
public:
    virtual void createKernel(vector<vector<double> > *spatial);

    double sigmaX() const;
    double sigmaY() const;
    double k() const;
    double phi() const;
    double theta() const;

public slots:
    void setSigmaX(double sigmaX);
    void setSigmaY(double sigmaY);
    void setK(double k);
    void setPhi(double phi);
    void setTheta(double theta);

signals:
    void sigmaXChanged(double sigmaX);
    void sigmaYChanged(double sigmaY);
    void kChanged(double k);
    void phiChanged(double phi);
    void thetaChanged(double theta);

private:
    double advance(int x, int y);
    double m_sigmaX;
    double m_sigmaY;
    double m_k;
    double m_phi;
    double m_theta;
};

#endif // GABORKERNELENGINE_H
