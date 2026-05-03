**MENU **"Info"

    "&Open Log"    ""    "StdLog.Open"    ""

    "&Clear Log"    ""    "StdLog.Clear"    ""

    **SEPARATOR**

    "&Loaded Modules"    ""    "DevDebug.ShowLoadedModules"    ""

    "&Global Variables"    ""    "DevDebug.ShowGlobalVariables"    "TextCmds.SelectionGuard"

    "&View State"    ""    "DevDebug.ShowViewState"    "StdCmds.SingletonGuard"

    "About &Alien"    ""    "DevAlienTool.Analyze"    "StdCmds.SingletonGuard"

    "&Heap Spy..."    ""    "StdCmds.OpenToolDialog('Dev/Rsrc/HeapSpy', 'Heap Spy')"    ""

    "Message Spy..."    ""    "DevMsgSpy.OpenDialog('Dev/Rsrc/MsgSpy', 'Message Spy')"    ""

    "Control List"    ""    "DevCmds.ShowControlList"    "StdCmds.ContainerGuard"

    **SEPARATOR**

    "&Source"    ""    "DevReferences.ShowSource"    "TextCmds.SelectionGuard"

    "Client Interface"    "D"    "DevBrowser.ShowInterface('@c')"    "TextCmds.SelectionGuard"

    "Extension Interface"    "*D"    "DevBrowser.ShowInterface('@e')"    "TextCmds.SelectionGuard"

    "Interface..."    ""    "StdCmds.OpenToolDialog('Dev/Rsrc/Browser', 'Browser')"    ""

    "&Documentation"    ""    "DevReferences.ShowDocu"    "TextCmds.SelectionGuard"

    "De&pendencies"    ""    "DevDependencies.Deposit;StdCmds.Open"    "TextCmds.SelectionGuard"

    "Create Tool"    ""    "DevDependencies.CreateTool"    "TextCmds.SelectionGuard"

    "&Repository"    ""    "DevRBrowser.ShowRepository"    ""

    **SEPARATOR**

**    **"Search In Sources"    ""    "TextCmds.InitFindDialog; DevSearch.SearchInSources"    "TextCmds.SelectionGuard"

    "Search In Docu (Case Sensitive)"    ""    "TextCmds.InitFindDialog; DevSearch.SearchInDocu('s')"    "TextCmds.SelectionGuard"

    "Search In Docu (Case Insensitive)"    ""    "TextCmds.InitFindDialog; DevSearch.SearchInDocu('i')"    "TextCmds.SelectionGuard"

    "Compare Texts"    "F9"    "DevSearch.Compare"    "TextCmds.FocusGuard"

    "Check Links..."    ""    "StdCmds.OpenToolDialog('Dev/Rsrc/LinkChk', 'Check Links')"    ""

    "Analyzer Options..."    ""    "StdCmds.OpenToolDialog('Dev/Rsrc/Analyzer', 'Analyze')"    ""

    "Analyze Module"    ""    "DevAnalyzer.Analyze"    "TextCmds.FocusGuard"

    **SEPARATOR**

    "&Menus"    ""    "StdMenuTool.ListAllMenus"    ""

    "&Update Menus"    ""    "StdMenuTool.UpdateAllMenus"    ""

**END**

**MENU **"Dev"

    "&Edit Mode"    ""    "StdCmds.SetEditMode"    "StdCmds.SetEditModeGuard"

    "&Layout Mode"    ""    "StdCmds.SetLayoutMode"    "StdCmds.SetLayoutModeGuard"

    "&Browser Mode"    ""    "StdCmds.SetBrowserMode"    "StdCmds.SetBrowserModeGuard"

    "&Mask Mode"    ""    "StdCmds.SetMaskMode"    "StdCmds.SetMaskModeGuard"

    **SEPARATOR**

    "&Open Module List"    "0"    "DevCmds.OpenModuleList"    "TextCmds.SelectionGuard"

    "Open &File List"    ""    "DevCmds.OpenFileList"    "TextCmds.SelectionGuard"

    **SEPARATOR**

    "&Compile"    "K"    "DevCompiler.Compile"    "TextCmds.FocusGuard"

    "Compile And Unload"    ""    "DevCompiler.CompileAndUnload"    "TextCmds.FocusGuard"

    "Compile &Selection"    ""    "DevCompiler.CompileSelection"    "TextCmds.SelectionGuard"

    "Com&pile Module List"    ""    "DevCompiler.CompileModuleList"    "TextCmds.SelectionGuard"

    **SEPARATOR**

    "Unmar&k Errors"    ""    "DevMarkers.UnmarkErrors"    "TextCmds.FocusGuard"

    "Next E&rror"    "E"    "DevMarkers.NextError"    "TextCmds.FocusGuard"

    "To&ggle Error Mark"    "T"    "DevMarkers.ToggleCurrent"    "TextCmds.FocusGuard"

    **SEPARATOR**

    "E&xecute"    ""    "DevDebug.Execute"    "TextCmds.SelectionGuard"

    "&Unload"    ""    "DevDebug.Unload"    "TextCmds.FocusGuard"

    "Unloa&d Module List"    ""    "DevDebug.UnloadModuleList"    "TextCmds.SelectionGuard"

    "Flus&h Resources"    ""    "DevCmds.FlushResources"    ""

    "Re&validate View"    ""    "DevCmds.RevalidateView"    "DevCmds.RevalidateViewGuard"

    **SEPARATOR**

    "Set Profile List"    ""    "DevProfiler.SetProfileList"    "DevProfiler.StartGuard"

    "Start Profiler"    ""    "DevProfiler.Start"    "DevProfiler.StartGuard"

    "Stop Profiler"    ""    "DevProfiler.Stop; DevProfiler.ShowProfile"    "DevProfiler.StopGuard"

    "Timed Execute"    ""    "DevProfiler.Execute"    "TextCmds.SelectionGuard"

**END**

**MENU **"Tools"

    "Document Size..."    ""    "StdCmds.InitLayoutDialog; StdCmds.OpenToolDialog('Std/Rsrc/Cmds1', 'Document Size')"

                "StdCmds.WindowGuard"

    "View Size..."     ""    "StdViewSizer.InitDialog;StdCmds.OpenToolDialog('Std/Rsrc/ViewSizer', 'View Size')"

                "StdCmds.SingletonGuard"

    **SEPARATOR**

    "Insert OLE &Object..."    ""    "OleClient.InsertObject"    "StdCmds.PasteViewGuard"

    "Insert Co&mmander"    "Q"    "DevCommanders.Deposit; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    "Insert &Stamp"    ""    "StdStamps.Deposit; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    "Insert Cloc&k"    ""    "StdClocks.Deposit; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    "Insert &Header"    ""    "StdHeaders.Deposit; StdCmds.PasteView; TextCmds.ShowMarks"    "TextCmds.FocusGuard"

    **SEPARATOR**

    "&Add Scroller"    ""    "StdScrollers.AddScroller"    "StdCmds.SingletonGuard"

    "&Remove Scroller"    ""    "StdScrollers.RemoveScroller"    "StdCmds.SingletonGuard"

    **SEPARATOR**

    "Create &Link"    "L"    "StdLinks.CreateLink"    "StdLinks.CreateGuard"

    "Create &Target"    ""    "StdLinks.CreateTarget"    "StdLinks.CreateGuard"

    **SEPARATOR**

    "Create Fold"    ""    "StdFolds.Create(1)"    "StdFolds.CreateGuard"

    "Expand All"    ""    "StdFolds.Expand"    "TextCmds.FocusGuard"

    "Collapse All"    ""    "StdFolds.Collapse"    "TextCmds.FocusGuard"

    "Fold..."    ""    "StdCmds.OpenToolDialog('Std/Rsrc/Folds', 'Zoom')"    ""

    **SEPARATOR**

    "Encode Document"    ""    "StdCoder.EncodeDocument"    "StdCmds.WindowGuard"

    "Encode Selection"    ""    "StdCoder.EncodeSelection"    "TextCmds.SelectionGuard"

    "Encode File..."    ""    "StdCoder.EncodeFile"    ""

    "Encode File List"    ""    "StdCoder.EncodeFileList"    "TextCmds.SelectionGuard"

    "Decode"    ""    "StdCoder.Decode"    "TextCmds.FocusGuard"

    "About Encoded Material"    ""    "StdCoder.ListEncodedMaterial"    "TextCmds.FocusGuard"

    **SEPARATOR**

    "Create Subsystem..."    ""    "StdCmds.OpenToolDialog('Dev/Rsrc/Create', 'Create Subsystem')"    ""

**END**

**MENU **"Controls"

    "&New Form..."    ""    "StdCmds.OpenToolDialog('Form/Rsrc/Gen', 'New Form')"    ""

    "Open As &Tool Dialog"    ""    "StdCmds.OpenAsToolDialog"    "StdCmds.ContainerGuard"

    "Open As &Aux Dialog"    ""    "StdCmds.OpenAsAuxDialog"    "StdCmds.ContainerGuard"

    **SEPARATOR**

    "Insert Tab &View"    ""    "StdTabViews.Deposit; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    **SEPARATOR**

    "Insert Co&mmand Button"    ""    "Controls.DepositPushButton; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    "Insert Chec&k Box"    ""    "Controls.DepositCheckBox; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    "Insert &Radio Button"    ""    "Controls.DepositRadioButton; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    "Insert &Edit Field"    ""    "Controls.DepositField; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    "Insert &List Box"    ""    "Controls.DepositListBox; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    "Insert &Selection Box"    ""    "Controls.DepositSelectionBox; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    "Insert Com&bo Box"    ""    "Controls.DepositComboBox; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    "Insert &Up/Down Field"    ""    "Controls.DepositUpDownField; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    "Insert &Time Field"    ""    "Controls.DepositTimeField; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    "Insert &Date Field"    ""    "Controls.DepositDateField; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    "Insert C&olor Field"    ""    "Controls.DepositColorField; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    "Insert Tree Control"    ""    "Controls.DepositTreeControl; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    "Insert Table Control"    ""    "StdTables.DepositControl; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    **SEPARATOR**

    "Insert &Cancel Button"    ""    "Controls.DepositCancelButton; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    "Insert Captio&n"    ""    "Controls.DepositCaption; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    "Insert &Group Box"    ""    "Controls.DepositGroup; FormCmds.InsertAround; FormCmds.SetAsFirst"    "StdCmds.PasteViewGuard"

**END**

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
