**Overview by Example: ObxFileTree**

This example illustrates the use of the *TreeControl*. The aim is to create a file browser similar to the Windows Explorer. To keep it simple, the user interface will consist of a text field, three buttons, and a *TreeControl*. When the user types a path to a directory in the text field and clicks on the *Display* button the files and folders in the given directory are shown in the tree. The user can browse the files and folders in the tree, and a double click on a file opens the file in BlackBox.

**The Interactors**

To achieve this user interface we need a *Dialog.String* variable to connect to the text field, we need two procedures to connect to the *Display* and *Open* buttons respectively, and we need a *Dialog.Tree* variable to connect to the *TreeControl*. We also need a notifier to connect to *TreeControl*. This notifier should detect a double click on a leaf in the tree and open the corresponding file.

The *TreeControl* has several properties which can be modified with the property inspector. For this example the default properties are sufficient. In particular the option "Folder icons" should be switched on. This option makes the *TreeControl* show icons in front of each node in the tree. The leaf nodes get a "file" icon and the nodes that have subnodes get a "folder" icon. This option gives our program the same look and feel as the Windows Explorer.

The only problem is that if a file directory does not contain any files it will be a leaf node and thus look like a file and not a folder. For this purpose the *Dialog.TreeNode* offers a method called *ViewAsFolder*, which makes a node in a tree display a folder icon even if it doesn't have any subnodes.

**The Implemantation**

To implement this, this example uses four procedures:

PROCEDURE BuildDirTree (loc: Files.Locator; parent: Dialog.TreeNode);

PROCEDURE **Update***;

PROCEDURE **Open***;

PROCEDURE **OpenGuard*** (VAR par: Dialog.Par);

*BuildDirTree* recursively adds files and folders to the tree, and also makes sure that the *ViewAsFolder* attribute is set for nodes that are folders. *Update* is the procedure that is connected to the *Display* command button in the user interface. This procedure clears the tree and then calls *BuildDirTree*. The procedure *Open *opens the selected folder or file. StdCmds.DefaultOnDoubleClick, which is set as the *TreeControl*'s notifier procedure, executes the default command *Open* when the *TreeControl* is double-clicked.

 "StdCmds.OpenToolDialog('Obx/Rsrc/FileTree', 'ObxFileTree Demo')"

[<u>ObxFileTree  sources</u>](../Mod/FileTree.odc.md)

