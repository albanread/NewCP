**DevDependencies**

DEFINITION DevDependencies;

    IMPORT Dialog;

    PROCEDURE ArrangeClick;

    PROCEDURE CollapseAllClick;

    PROCEDURE CollapseClick;

    PROCEDURE CreateTool;

    PROCEDURE CreateToolClick;

    PROCEDURE Deposit;

    PROCEDURE ExpandAllClick;

    PROCEDURE ExpandClick;

    PROCEDURE HideClick;

    PROCEDURE ModsGuard (VAR par: Dialog.Par);

    PROCEDURE NewAnalysisClick;

    PROCEDURE SelGuard (VAR par: Dialog.Par);

    PROCEDURE ShowAllClick;

    PROCEDURE ShowBasicGuard (VAR par: Dialog.Par);

    PROCEDURE SubsGuard (VAR par: Dialog.Par);

    PROCEDURE ToggleBasicSystemsClick;

END DevDependencies.

This tool analyzes the code files of a given list of modules. The dependencies from the given modules to other modules are displayed in a graph. By default all selected modules are displayed, but all other modules are only displayed as subsystems. Clicking with the right mouse button displays a context menu. This menu contains several commands to manipulate the view of the graph.

A subsystem can be expanded by double-clicking on it. In the same way, all modules belonging to a particular subsystem can be collapsed into a subsystem by double-clicking on any of the modules in the subsystem.

By clicking on a node in the graph this node gets selected. It is possible to select more than one node by *Ctrl-clicking* on other nodes or by "drawing" a selection square around some nodes. After nodes have been selected they can be rearranged using drag and drop with the mouse or using the arrow keys on the keyboard. *Ctrl-A* can be used to select all nodes.

To avoid cluttering of the graph, the basic BlackBox subsystems are hidden by default. It is however possible to toggle an option to show these modules.

There are several implicit dependencies in the system (e.g., via *Dialog.Call*). These dependencies cannot be found using the code files. To be able to incorporate these dependencies, at least in a static way, they can be specified in the string resources for the *Dev* subsystem. The syntax for specifying implicit dependencies is:

Implicit.<modName>    <modname>{, <modname>}

*DevDependencies* reads this resource and adds these dependencies to the graph. Such dependencies are displayed as gray arrows in the graph.

It is also possible to create a tool document using the command *CreateTool*. This creates a tool document which contains compiling, unloading, linking and packing commands for the given modules. The compiling and unloading commands are only created for the currently expanded subsystems. The linking command includes the standard BlackBox icons, and the packing command always includes all modules independent of which modules are currently expanded or hidden.

In the created tool document all the modules, which are only included due to implicit dependencies are written with gray color.

At the end of the tool document there is a list of all the root modules. These are the modules, which are not imported from any other modules, i.e. the top level modules.

Typical menu:

**MENU**

    "&Dependencies"    ""    "DevDependencies.Deposit;StdCmds.Open"    "TextCmds.SelectionGuard"

    "&Create Tool"    ""    "DevDependencies.CreateTool"    "TextCmds.SelectionGuard"

**END**

By clicking on the right mouse button a context menu is displayed. Typically it has the following entries:

**MENU** "*" ("DevDependencies.View")

    "Expand"    ""    "DevDependencies.ExpandClick"    "DevDependencies.SubsGuard"

    "Collapse"    ""    "DevDependencies.CollapseClick"    "DevDependencies.ModsGuard"

    "New Analysis"    ""    "DevDependencies.NewAnalysisClick"    "DevDependencies.ModsGuard"

    "Hide"    ""    "DevDependencies.HideClick"    "DevDependencies.SelGuard"

    SEPARATOR

    "Show All Items"    ""    "DevDependencies.ShowAllClick"    ""

    "Show Basic System"    ""    "DevDependencies.ToggleBasicSystemsClick"    "DevDependencies.ShowBasicGuard"

    "Expand All"    ""    "DevDependencies.ExpandAllClick"    ""

    "Collapse All"    ""    "DevDependencies.CollapseAllClick"    ""

    "Arrange Items"    ""    "DevDependencies.ArrangeClick"    ""

    SEPARATOR

    "Create tool..."    ""    "DevDependencies.CreateToolClick"    ""

    SEPARATOR

    "P&roperties..."    ""    "StdCmds.ShowProp"    "StdCmds.ShowPropGuard"

**END**

The item called *Properties* in this menu opens the standard font dialog and lets the user select a font for the view. The typeface and style of the chosen font effects the text displaying the names of the modules.The size of the font effects the text and the width of the lines and the arrows.

PROCEDURE **Deposit**

Analyzes the dependencies for each module in the selected list of modules. Then a view, displaying a graph of the dependencies, is created and deposited.

PROCEDURE **CreateTool**

Analyzes the dependencies for each module in the selected list of modules. Then a tool document is created for the given modules. The tool document contains compiling, unloading, linking and packing commands. The compiling and unloading commands are only created for the currently expanded modules. The linking command includes the standard BlackBox icons and the packing command always includes all modules independent of which modules are currently expanded or hidden.

The following commands are only used for manipulating the view (ordered the way they appear in the menu above):

PROCEDURE **CollapseClick**

Called when *Collapse* is chosen from the menu. This procedure collapses the the selected modules in the graph.

PROCEDURE **ExpandClick**

Called when *Expand* is chosen from the menu. This procedure expands the the selected subsystems in the graph.

PROCEDURE **NewAnalysisClick**

Called when *New Analysis* is chosen from the menu. Starts a new analysis with the selected node as root, i.e. it creates a subgraph of the original graph.

PROCEDURE **HideClick**

Called when *Hide* is chosen from the menu. This procedure hides the selected nodes in the graph.

PROCEDURE **ShowAllClick**

Called when *Show All Items* is chosen from the menu. This procedure makes sure that all nodes in the graph are visible.

PROCEDURE **ToggleBasicSystemsClick**

Called when *Show Basic System* is chosen from the menu. This procedure shows or hides modules belonging to the basic BlackBox system.

PROCEDURE **ExpandAllClick**

Called when *Expand All* is chosen from the menu. This procedure expands all subsystems to modules.

PROCEDURE **CollapseAllClick**

Called when *Collapse All* is chosen from the menu. This procedure collapses all modules into subsystems.

PROCEDURE **ArrangeClick**

Called when *Arrange Items* is chosen from the menu. This procedure arranges the nodes in the graph in a structured way, with the modules from the original module list at the top.

PROCEDURE **CreateToolClick**

Called when *Create tool* is chosen from the menu. Calls *CreateTool*.

PROCEDURE **SelGuard** (VAR par: Dialog.Par)

Guard that checks if any nodes in the graph are selected.

PROCEDURE **ModsGuard** (VAR par: Dialog.Par)

Guard that checks if any modules in the graph are selected.

PROCEDURE **SubsGuard** (VAR par: Dialog.Par)

Guard that checks if any subsystems in the graph are selected.

PROCEDURE **ShowBasicGuard** (VAR par: Dialog.Par)

Guard for the *Show Basic System* menu item.
