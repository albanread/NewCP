**MENU** "Obx"

    "Hello0"    ""    "ObxHello0.Do"    ""

    "Hello1"    ""    "ObxHello1.Do"    ""

    "Open0..."    ""    "ObxOpen0.Do"    ""

    "Open1..."    ""    "ObxOpen1.Do"    ""

    "Capitalize Text"    ""    "ObxCaps.Do"    "TextCmds.SelectionGuard"

    "Open Mail Template"    ""    "StdCmds.OpenDoc('Obx/Samples/MMTmpl')"    ""

    "Merge..."    ""    "ObxMMerge.Merge"    "TextCmds.FocusGuard"

    "Show Directory"    ""    "ObxLinks.Directory('')"    ""

    **SEPARATOR**

    "Orders..."    ""    "StdCmds.OpenAuxDialog('Obx/Rsrc/Orders', 'Order Processing')"    ""

    "Controls..."    ""    "StdCmds.OpenAuxDialog('Obx/Rsrc/Controls', 'ObxControls Demo')"    ""

    "Dialog..."    ""    "StdCmds.OpenAuxDialog('Obx/Rsrc/Dialog', 'ObxDialog Demo')"    ""

    "File Tree..."    ""    "StdCmds.OpenAuxDialog('Obx/Rsrc/FileTree', 'ObxFileTree Demo')"    ""

    "Tab View..."    ""    "ObxTabViews.Deposit; StdCmds.Open"    ""

    **SEPARATOR**

    "Trap!"    ""    "ObxTrap.Do"    ""

    "Primes..."    ""    "StdCmds.OpenAuxDialog('Obx/Rsrc/Actions', 'Prime Calculation')"    ""

    "Compute Factorial"    ""    "ObxFact.Compute"    "TextCmds.SelectionGuard"

    "Simplify"    ""    "ObxRatCalc.Simplify"    "TextCmds.SelectionGuard"

    "Approximate"    ""    "ObxRatCalc.Approximate"    "TextCmds.SelectionGuard"

    **SEPARATOR**

    "New Pattern"    ""    "ObxPatterns.Deposit; StdCmds.Open"    ""

    "New Calculator"    ""    "ObxCalc.Deposit; StdCmds.Open"    ""

    "New Omosi"    ""    "ObxOmosi.Deposit; StdCmds.Open"    ""

    "New Cube"    ""    "ObxCubes.Deposit; StdCmds.Open"    ""

    "Cube Colors..."    ""    "StdCmds.OpenToolDialog('Obx/Rsrc/Cubes', 'Cube Colors')"    ""

    "New Checkerboard"    ""    "ObxScroll.Deposit; StdCmds.Open"    ""

    **SEPARATOR**

    "Insert Button"    ""    "ObxButtons.Deposit; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    "New Ticker"    ""    "ObxTickers.Deposit; StdCmds.Open"    ""

    **SEPARATOR**

    "New Lines"    ""    "ObxLines.Deposit; StdCmds.Open"    ""

    "New Graph"    ""    "ObxGraphs.Deposit; StdCmds.Open"    ""

    "Black Box..."    ""    "StdCmds.OpenAuxDialog('Obx/Rsrc/BlackBox', 'BlackBox')"    ""

    **SEPARATOR**

    "Wrap"    ""    "ObxWrappers.Wrap"    "StdCmds.SingletonGuard"

    "Unwrap"    ""    "ObxWrappers.Unwrap"    "StdCmds.SingletonGuard"

    **SEPARATOR**

    "New Twin"    ""    "ObxTwins.Deposit; StdCmds.Open"    ""

    "Focus Magic Control"    ""    "ObxContIter.Do"    "StdCmds.ContainerGuard"

END

**MENU** "Tut"

    "Phone Database..."    ""    "StdCmds.OpenAuxDialog('Obx/Rsrc/PhoneUI', 'Phone Database')"    ""

    "Phone Database 1..."    ""    "StdCmds.OpenAuxDialog('Obx/Rsrc/PhoneUI1', 'Phone Database')"    ""

    "Right-Shift Selection"    ""    "ObxControlShifter.Shift"    "FormCmds.SelectionGuard"

    "List Labels"    ""    "ObxLabelLister.List"    "FormCmds.FocusGuard"

    "Generate Report 0"    ""    "ObxPDBRep0.GenReport"    ""

    "Generate Report 1"    ""    "ObxPDBRep1.GenReport"    ""

    "Generate Report 2"    ""    "ObxPDBRep2.GenReport"    ""

    "Generate Report 3"    ""    "ObxPDBRep3.GenReport"    ""

    "Generate Report 4"    ""    "ObxPDBRep4.GenReport"    ""

    "Count Atoms"    ""    "ObxCount0.Do"    "TextCmds.SelectionGuard"

    "Count Symbols"    ""    "ObxCount1.Do"    "TextCmds.SelectionGuard"

    "Lookup 0"    ""    "ObxLookup0.Do"    "TextCmds.SelectionGuard"

    "Lookup 1"    ""    "ObxLookup1.Do"    "TextCmds.SelectionGuard"

    **SEPARATOR**

    "New View 0"    ""    "ObxViews0.Deposit; StdCmds.Open"    ""

    "New View 1"    ""    "ObxViews1.Deposit; StdCmds.Open"    ""

    "New View 2"    ""    "ObxViews2.Deposit; StdCmds.Open"    ""

    "New View 3"    ""    "ObxViews3.Deposit; StdCmds.Open"    ""

    "New View 4"    ""    "ObxViews4.Deposit; StdCmds.Open"    ""

    "New View 5"    ""    "ObxViews5.Deposit; StdCmds.Open"    ""

    "New View 6"    ""    "ObxViews6.Deposit; StdCmds.Open"    ""

    **SEPARATOR**

    "New View 10"    ""    "ObxViews10.Deposit; StdCmds.Open"    ""

    "New View 11"    ""    "ObxViews11.Deposit; StdCmds.Open"    ""

    "New View 12"    ""    "ObxViews12.Deposit; StdCmds.Open"    ""

    "New View 13"    ""    "ObxViews13.Deposit; StdCmds.Open"    ""

    "New View 14"    ""    "ObxViews14.Deposit; StdCmds.Open"    ""

END

**MENU** "New" ("Obx.Tutorial")

    "Beep"    ""    "Dialog.Beep"    ""

END

**MENU** "BlackBox" ("ObxBlackBox.View")

    "Show Solution"    ""    "ObxBlackBox.ShowSolution"    "ObxBlackBox.ShowSolutionGuard"

    **SEPARATOR**

    "New Atoms"    ""    "ObxBlackBox.New"    ""

    "Set New Atoms"    ""    "ObxBlackBox.Set"    ""

    **SEPARATOR**

    "Rules"    ""    "StdCmds.OpenBrowser('Obx/Docu/BB-Rules', 'BlackBox Rules')"    ""

**END**

