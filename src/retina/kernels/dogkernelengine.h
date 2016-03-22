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
};

#endif // DOGKERNELENGINE_H
