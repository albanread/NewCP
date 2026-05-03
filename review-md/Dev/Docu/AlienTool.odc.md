**DevAlienTool**

DEFINITION DevAlienTool;

    PROCEDURE Analyze;

END DevAlienTool.

Alien views are views which, for some reason, cannot be loaded correctly. The reason may be a programming error, e.g. a different number of bytes is read than was written, or it may be a version problem (an unknown version was detected). Usually however, the problem is that some code for the view cannot be loaded, because it is missing or inconsistent. The alien tool helps to analyze such problems.

PROCEDURE **Analyze**

Guard: StdCmds.SingletonGuard

This command analyzes the singleton alien view, and opens a window with the textual analysis.
