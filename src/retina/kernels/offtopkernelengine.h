#ifndef OFFTOPKERNELENGINE_H
#define OFFTOPKERNELENGINE_H

#include "abstractkernelengine.h"



class OffTopKernelEngine : public AbstractKernelEngine
{
public:
    OffTopKernelEngine();

    // AbstractKernelEngine interface
public:
    virtual void createKernel(vector<vector<double> > *spatial);
};

#endif // OFFTOPKERNELENGINE_H
