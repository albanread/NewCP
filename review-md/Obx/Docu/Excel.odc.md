**Overview by Example: ObxExcel**

This example how the OLE Automation controller subsystem can be used to program Microsoft Excel. Use the following menu entries to run the examples:

    "Show Excel"    ""    "ObxExcel.ShowExcel"    ""

    "Read Excel"    ""    "ObxExcel.ReadExcel"    "StdCmds.SingletonGuard"

    "Open Chart"    ""    "ObxExcel.OpenChart"    "StdCmds.SingletonGuard"

ShowExcel adds different values to a new worksheet and opens the worksheet as an OLE object in a BlackBox text.

ReadExcel reads a selected worksheet object and writes the contents of the cells into the log.

OpenChart uses the values in a selected worksheet object and opens a corresponding chart in a separate Excel window.

See also the [<u>Ctl Developer Manual</u>](../../Ctl/Docu/Dev-Man.odc.md).

[<u>ObxExcel  sources</u>](../Mod/Excel.odc.md)

