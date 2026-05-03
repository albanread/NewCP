**OleViews**

DEFINITION OleViews;

    IMPORT CtlT, Views;

    PROCEDURE NewObjectView (name: ARRAY OF CHAR): Views.View;

    PROCEDURE NewObjectViewFromClipboard (): Views.View;

    PROCEDURE Deposit (name: ARRAY OF CHAR);

    PROCEDURE IsObjectView (v: Views.View): BOOLEAN;

    PROCEDURE IsAutoView (v: Views.View): BOOLEAN;

    PROCEDURE OleObject (v: Views.View): CtlT.Interface;

    PROCEDURE AutoObject (v: Views.View): CtlT.Object;

    PROCEDURE Connect (sink: CtlT.OutObject; source: Views.View);

END OleViews.

*OleViews* provides a programming interface for OLE objects used as views in BlackBox.

PROCEDURE **NewObjectView** (name: ARRAY OF CHAR): Views.View

Instantiates an OLE object of a given type. *name* can either be a valid OLE class name (e.g., "Excel.Sheet") or a GUID (global unique identifier) in string form, provided it is a valid class identifier (e.g., "{00020820-0000-0000-C000-000000000046}"). If instatiantion failed, the return value is *NIL*.

PROCEDURE **NewObjectViewFromClipboard** (): Views.View

Instantiates the OLE object currently contained in the clipboard. Returns NIL if there is no OLE object on the clipboard.

PROCEDURE **Deposit** (name: ARRAY OF CHAR)

Deposit command for new OLE objects of a given type. *name* can either be a valid OLE class name (e.g., "Excel.Sheet") or a GUID (global unique identifier) in string form, provided it is a valid class identifier (e.g., "{00020820-0000-0000-C000-000000000046}").

PROCEDURE **IsObjectView** (v: Views.View): BOOLEAN

Return *TRUE* if the given View is an OLE object, *FALSE* otherwise.

PROCEDURE **IsAutoView** (v: Views.View): BOOLEAN

Return *TRUE* if the given View is an OLE Automation object (an object which supports the *IDispatch* interface), *FALSE* otherwise.

PROCEDURE **OleObject** (v: Views.View): CtlT.Interface

Returns the *IUnknown* interface of the object contained in *v* as an automation controller object. Only useful in conjunction with the Direct-To-COM compiler (DTC). For information about automation controller objects see the [<u>Ctl Docu</u>](../../Ctl/Docu/Dev-Man.odc.md). *NIL* is returned if *v* is not an OLE object.

PROCEDURE **AutoObject** (v: Views.View): CtlT.Object

Returns the automation controller object associated with *v.* For information about automation controller objects see the [<u>Ctl Docu</u>](../../Ctl/Docu/Dev-Man.odc.md). *NIL* is returned if *v* is not an OLE Automation object.

PROCEDURE **Connect** (sink: CtlT.OutObject; source: Views.View)

Connects the automation controller callback object *sink* to an OLE object contained in the view *source*. For information about automation controller objects see the [<u>Ctl Docu</u>](../../Ctl/Docu/Dev-Man.odc.md).

