**Overview by Example: ObxBlackBox**

This example implements the deductive game *BlackBox*, using the *BlackBox* Component Framework. The aim of the game is to guess the positions of atoms within an *n* by *n* grid by firing rays into a black box. The rays may be deflected or even absorbed. From the position where the ray leaves the grid one must try to deduce the position of the atoms. For more information about *BlackBox* and on how to play the game read the [<u>rules</u>](BB-Rules.odc.md).

A main point of this example is to demonstrate one advantage of compound documents: The *ObxBlackBox* views which are implemented can directly be used in the documentation for illustration purposes. Neither additional code nor the use of a drawing program is required. Moreover, these views in the documentation are living elements and can be inspected and modified as desired.

This example also shows how a view-specific menu can be used. If a *BlackBox* view is the focus view, a special menu appears which offers *BlackBox*-specific commands that operate on the focus view. For this purpose, the following menu is installed in your Obx menu file (-> [<u>Menus</u>](../Rsrc/Menus.odc.md)):

The first menu entry is enabled or disabled depending on the state of the focus view. A user-defined guard is added for this purpose. For more information about the commands themselves see the section on how to play *BlackBox* in the [<u>rules</u>](BB-Rules.odc.md).

 "StdCmds.OpenAuxDialog('Obx/Rsrc/BlackBox', 'BlackBox')"

[<u>ObxBlackBox  sources</u>](../Mod/BlackBox.odc.md)

