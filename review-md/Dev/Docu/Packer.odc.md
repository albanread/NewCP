**DevPacker**

DEFINITION DevPacker;

    PROCEDURE ListFromSub (sdir: ARRAY OF CHAR);

    PROCEDURE ListLoadedModules;

    PROCEDURE PackThis;

END DevPacker.

Module *DevPacker* is used to pack any kind of files into an existing exe-file. These files can be read with the help of *HostPackedFiles*. There is no explicit dependency between *DevPacker* and *HostPackedFiles*, but since one writes to the exe-file and the other reads from it, there is an implicit dependency, with the file format of the exe-file as interface.

*DevDependencies.CreateTool* can be used to create a pack command for a set of modules.

PROCEDURE **PackThis**;

Used together with a *DevCommander.* It reads a list of file names and packs the given files into an exe-file. The exe-file appears first in the list followed by the symbol ":=". It is also possible to pack a file under a different name than it appears in the current filesystem. To specify another name for a file the special symbol "=>" is used.

Thus, the syntax in EBNF is as follows:

<exeFileName> := <filename> [=> <filename>] {<filename> [=> <filename>]}

Both the *exeFileName* and the *fileName* can be specified using absolute paths (including drive name) or using relative paths to the BlackBox directory. If any file contains special characters in their name, such as space or dash, then the file name should be embedded in quotation marks ("filename") to be parsed correctly. The parsing stops as soon as it encounters any unrecognized character such as a comma or a tilde.

Before the packing starts the list of files are validated. If some files are not found, then the package command is discontinued and the exe-file is left untouched. If a file name appears more than once in the list, a message that the file appears more than once is presented but the package command is continued and the file is packed just once into the exe-file.

It is important that the exe-file does not appear in the list of files to be packed. This would cause the exe-file to be packed into itself with undefined result. Therefore the packing is cacelled if the exe-file appears in the list.

Example:

 DevPacker.PackThis exefilename.exe :=

Tour.odc "C:\Program files\BlackBox\License.txt" Test/Code/MyConfig.ocf => Code/Config.ocf

PROCEDURE **ListFromSub** (sdir: ARRAY OF CHAR);

Examines the directory *sdir* and all its subdirectories. If it finds any files it opens a new document and writes a packing command for the files found. *sdir* can be an absolute path, including a drive name, or a relative path to the BlackBox directory. It can also be an empty string, in which case the entire BlackBox directory will be listed.

Example:  "DevPacker.ListFromSub('Text')"

PROCEDURE **ListLoadedModules**;

Examines the currently loaded modules and creates a text with a packing command for them. This can be used to quickly find out which modules are needed for a running application. The problem is that too many modules may be packed and that files such as resources and documentaion are not included.

Example:  DevPacker.ListLoadedModules

**Absolute vs. Relative paths in filenames**

Imagine that BlackBox is installed in directory *D:\BlackBox*. Then the file name *D:\BlackBox\Std\Code\Log.ocf* (absolute path) denotes the same file as *Std\Code\Log.ocf* (relative path) as far as the packer is concerned. But the packer packs the file into the exe-file with the path given in the command. This makes a crucial difference for *HostPackedFiles* which reads the exe-file. If the exe-file for example is started in directory *C:\temp*, and the program makes a call to *StdLog*, then this command will work if the file was packed using a relative path, but not if it was packed using an absolute path. On the other hand, if the file was packed using an absolute path, a call to open file *D:\BlackBox\Std\Code\Log.ocf* will work even if no physical drive called *D:* is present on the machine.

**Regarding exe-files**

The packer cannot create an exe-file, it can only pack files into an existing exe-file. This means that the linker has to be used to create a minimum exefile for the packer to use. This minimum exefile must have *HostPackedFiles* linked in since this is the module that allows for extraction of the files. The minimum linker command is as follows:

 DevLinker.Link exefilename.exe := Kernel$+ Files HostFiles HostPackedFiles StdLoader

For *HostPackedFiles* to be able to find files in the exe-file, it must have the same format as when the packer packed the files into it. This means that it is not possible to add resource, like icons and cursors, after files have been packed to an exe-file. Any such operations should be done before the packing starts. (See the [<u>DevLinker -documentation</u>](Linker.odc.md) for details about how to add icons already when linking.) It is however possible to change the name of the exe-file after the packing is done.

Another limitation is that the packer is not able to append files. It packs the whole list of files and writes a table of these files. If *PackThis* is called again it destroys the information about the former files in the exe-file.

