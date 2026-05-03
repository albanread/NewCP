MODULE ObxUnitConv;

*(***

*    project    = "BlackBox"*

*    organization    = "www.oberon.ch"*

*    contributors    = "Oberon microsystems"*

*    version    = "[**<u>System/Rsrc/About</u>*](StdCmds.OpenToolDialog('System/Rsrc/About', 'About BlackBox'))*"*

*    copyright    = "[**<u>System/Rsrc/About</u>*](StdCmds.OpenToolDialog('System/Rsrc/About', 'About BlackBox'))*"*

*    license    = "[**<u>Docu/BB-License</u>*](../../Docu/BB-License.odc.md)*"*

*    changes    = ""*

*    issues    = ""*

***)*

    IMPORT Ports, Dialog;

    VAR

        **par***: RECORD metric*, inch*, point*: REAL; metricUnit*: Dialog.List END;

        metricUnits: ARRAY 4 OF INTEGER;

    PROCEDURE **NotifyMetric*** (op, from, to: INTEGER);

        VAR units: REAL;

    BEGIN

        units := par.metric * metricUnits[par.metricUnit.index] * Ports.mm;

        par.inch := units / Ports.inch; par.point := units / Ports.point;

        Dialog.Update(par)

    END NotifyMetric;

    PROCEDURE **NotifyInch*** (op, from, to: INTEGER);

        VAR units: REAL;

    BEGIN

        units := par.inch * Ports.inch;

        par.metric := units / metricUnits[par.metricUnit.index] / Ports.mm; par.point := units / Ports.point;

        Dialog.Update(par)

    END NotifyInch;

    PROCEDURE **NotifyPoint*** (op, from, to: INTEGER);

        VAR units: REAL;

    BEGIN

        units := par.point * Ports.point;

        par.metric := units / metricUnits[par.metricUnit.index] / Ports.mm; par.inch := units / Ports.inch;

        Dialog.Update(par)

    END NotifyPoint;

BEGIN

    par.metricUnit.SetLen(4);

    par.metricUnit.SetItem(0, "mm"); metricUnits[0] := 1;

    par.metricUnit.SetItem(1, "cm"); metricUnits[1] := 10;

    par.metricUnit.SetItem(2, "m"); metricUnits[2] := 1000;

    par.metricUnit.SetItem(3, "km"); metricUnits[3] := 1000000

END ObxUnitConv.

