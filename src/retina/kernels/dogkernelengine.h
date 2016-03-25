#ifndef DOGKERNELENGINE_H
#define DOGKERNELENGINE_H

#include "abstractkernelengine.h"



class DogKernelEngine : public AbstractKernelEngine
{
public:
    DogKernelEngine();

    // AbstractKernelEngine interface
public:
    virtual void createKernel(vector<vector<double> > *spatial);
private:
    double advance(double x, double y);
};

#endif // DOGKERNELENGINE_H
