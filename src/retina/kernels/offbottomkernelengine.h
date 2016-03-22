#ifndef OFFBOTTOMKERNELENGINE_H
#define OFFBOTTOMKERNELENGINE_H

#include "abstractkernelengine.h"



class OffBottomKernelEngine : public AbstractKernelEngine
{
public:
    OffBottomKernelEngine();

    // AbstractKernelEngine interface
public:
    virtual void createKernel(vector<vector<double> > *spatial);
};

#endif // OFFBOTTOMKERNELENGINE_H
