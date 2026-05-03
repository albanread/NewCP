**Config**

DEFINITION Config;

    PROCEDURE Setup;

END Config.

BlackBox attempts to call the command *Config.Setup* during start-up. The call allows to customize the configuration of BlackBox. Module *Config* is provided in source form and can be changed by the programmer arbitrarily. The default implementation looks like this:

    MODULE Config;

        IMPORT Dialog;

        PROCEDURE **Setup***;

            VAR res: INTEGER;

        BEGIN

            (* ... various file and clipboard converters are installed here ... *)

            Dialog.Call("StdLog.Open", "", res)

        END Setup;

    END Config.

This configuration causes the *log* window to be opened upon startup. The command is called after the complete BlackBox library, framework, and standard subsystems are loaded. *Config* may import any BlackBox module. If it isn't needed, module *Config* can be deleted.

[<u>Current Configuration</u>](../Mod/Config.odc.md)

PROCEDURE **Setup**

This procedure can be implemented in order to customize the initial BlackBox configuration after start-up. It is called after all standard services and menus have been installed.

Implementing *Setup* is optional.

Example for the implementation of *Config.Setup*:

    PROCEDURE Setup*;

        VAR res: INTEGER;

    BEGIN

        Dialog.Call("StdCmds.OpenAuxDialog('System/Rsrc/About', 'Splash Screen')", "", res)

    END Setup;

If *Config.Setup* need not be implemented, the module (in particular its code file) may be deleted entirely.

Note that similar to *Config*, there may (but need not) be a module *Startup* with a command *Setup*. It has a similar purpose as *Config*, but is called before the higher levels (text subsystem, form subsystem, etc.) are loaded. Consequently, *Startup* may not import the higher levels of BlackBox. Normally, *Startup* does not exist. It is only used under special circumstances, e.g., to overwrite the variable *Dialog.appName*.

