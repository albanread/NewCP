**DevProfiler**

DEFINITION DevProfiler;

 PROCEDURE SetProfileList;

 PROCEDURE SetModuleList (list: ARRAY OF CHAR);

 PROCEDURE Start;

 PROCEDURE Stop;

 PROCEDURE ShowProfile;

 PROCEDURE Reset;



 PROCEDURE Execute;



 PROCEDURE StartGuard (VAR par: Dialog.Par);

 PROCEDURE StopGuard (VAR par: Dialog.Par);

END DevProfiler.

*DevProfiler* is a statistical profiler for Component Pascal programs. A profiler measures how much processing time is spent in the individual procedures of a program. A statistical profiler determines at regular time intervals (interrupt-driven) in which procedure of which module the program currently executes. These measurements are stored, and can later be used to display the profile.

Profiles can be taken over all modules, or over a selected list of particularly interesting modules. Profiling can be started and stopped interactively, or via a programming interface. The latter often allows a more precise measurement.

Possible menus:

**MENUS**

     "Set Profile List" "" "DevProfiler.SetProfileList" "DevProfiler.StartGuard"

     "Start Profiler" "" "DevProfiler.Start" "DevProfiler.StartGuard"

     "Stop Profiler" "" "DevProfiler.Stop; DevProfiler.ShowProfile" "DevProfiler.StopGuard"

     "Execute" "" "DevProfiler.Execute" "TextCmds.SelectionGuard"

**END**

**Programming Example**

The following example shows how the programming interface of the profiler can be used. It profiles the compilation of a module, i.e., of command *DevCompiler.Compile*.

     PROCEDURE **ProfiledCompile***;

        VAR res: INTEGER;

    BEGIN

        DevProfiler.SetModuleList("DevCPM DevCPS DevCPT DevCPB DevCPP DevHostCPL

    DevHostCPC DevHostCPV");

        DevProfiler.Start;

        Dialog.Call("DevCompiler.Compile", "", res);

        DevProfiler.Stop;

        DevProfiler.ShowProfile;

        DevProfiler.Reset

     END ProfiledCompile;

The above procedure produces something like the following output:



In this example, about 37% of the measured time period has been spent in some procedures of module *DevCPT*. Procedure *InsertImport* alone has taken about 7% of the time. Note that procedures which used less than 1% of the time are not displayed, thus the sum over all *DevCPT* procedures doesn't add up to 37%.

Some time may be spent in modules not measured, or not in Component Pascal modules at all, e.g., in the host operating system's file system implementation. For this reason, and because modules which used less than 1% of the time are not shown, the sum over all modules doesn't add up to 100%.

PROCEDURE **SetProfileList**

This procedure takes the list of selected module names and registers it. This list of modules will be profiled when *Start* is called. If there is no selection, all listed modules will be profiled.

PROCEDURE **SetModuleList** (list: ARRAY OF CHAR)

Same as *SetProfileList* but with an explicit parameter instead of the selection as implicit parameter. This procedure is useful when the profiler is called from a program rather than interactively.

PROCEDURE **Start**

Starts profiling.

PROCEDURE **Stop**

Stops profiling.

PROCEDURE **ShowProfile**

Displays the most recently measured profile in a new window.

PROCEDURE **Reset**

Releases memory used for the profiler, including the most recently measured profile and the module list.

PROCEDURE **Execute**

Same as *DevDebug.Execute*. The execution time (in milliseconds) is written to the log.

