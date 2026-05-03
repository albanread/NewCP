**Overview by Example: ObxHello0**

After you have started the BlackBox Component Builder, you can open the file *Hello0* in the *Docu* directory of the *Obx* directory. It contains exactly the text you are now reading.

A first example of a Component Pascal module is given below, as an embedded text object:



To compile this module, its source code must first be focused (i.e., set the caret or selection in it). This is done by clicking somewhere in the above source code. Then execute the *Compile* command in the *Dev* menu. After the compilation, a message like

compiling "ObxHello0"   104   0

appears in the *Log* window. It means that module *ObxHello0* has been compiled successfully, that a code file has been written to disk containing the compiled code, that this module has a code size of 104 bytes and global variables of 0 bytes size, and that information about the module's interface has been written to disk in a symbol file. New symbol files are created whenever a module is compiled for the first time, or when its interface has changed.

The interface of the above module consists of one exported procedure: *ObxHello0.Do*. In order to call this procedure, a commander can be used (the little button below):

   ObxHello0.Do

Click on this button to cause the following actions to occur: the code for *ObxHello0* is loaded into memory, and then its procedure *Do* is executed. You see the result in the log window:

Hello World

When you click on the button again, BlackBox executes the command immediately, without loading the module's code again: once loaded, modules remain loaded unless they are removed explicitly.

When clicked, a commander takes the string which follows it, and tries to interpret it as a Component Pascal command, i.e., as a module name followed by a dot, followed by the name of an exported, parameterless procedure.

Try out the following examples:

   StdLog.Clear

   Dialog.Beep

   DevDebug.ShowLoadedModules

The list of loaded modules can be inspected by executing the *Loaded Modules* command in the *Info* menu, or by simply clicking in the appropriate commander above. It will generate a text similar to the following:

*module name    bytes used    clients    compiled    loaded*    <u>Update</u>

ObxHello0       106       0    25.10.1994  20:07:08    25.10.1994  20:08:42

Out       532       1    24.10.1994  13:53:49    25.10.1994  20:08:42

Config       159       0    24.10.1994  13:53:47    25.10.1994  18:31:52

A module can be unloaded if its client count is zero, i.e., if it is not imported by any other module. In order to unload *ObxHello0*, focus its source text again and then execute *Unload* in the *Dev* menu. The following message will appear in the log window:

ObxHello0 unloaded

When you generate the list of loaded modules again, it will look as follows:

*module name    bytes used    clients         compiled         loaded    *<u>Update</u>

Out       532      0    24.10.1994  13:53:49    25.10.1994  20:08:42

Config       159      0    24.10.1994  13:53:47    25.10.1994  18:31:52

Note that a module list is just a text which shows a snapshot of the loader state at a given point in time, it won't be updated automatically when you load or unload modules. You can print the text, save it in a file, or edit it without danger that it may be changed by the system. You can force an update by clicking on the blue update link in the text.

In this first example, we have seen how a very simple module looks like, how it can be compiled, how its command can be executed, how commanders are used as convenient alternatives to menu entries during development, how the list of loaded modules can be inspected, and how a loaded module can be unloaded again.

