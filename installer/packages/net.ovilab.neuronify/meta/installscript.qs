function Component()
{
}

Component.prototype.createOperations = function()
{
    component.createOperations();
    if (systemInfo.productType === "windows") {
       component.addOperation("CreateShortcut", "@TargetDir@/neuronify.exe", "@StartMenuDir@/Neuronify.lnk",
                              "workingDirectory=@TargetDir@", "iconPath=@TargetDir@/neuronify.ico", "iconId=0");
    }
}
