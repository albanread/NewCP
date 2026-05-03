**Models**

DEFINITION Models;

    IMPORT Stores;

    CONST clean = 0; notUndoable = 1; invisible = 2;

    TYPE

        Model = POINTER TO ABSTRACT RECORD (Stores.Store)

            PROCEDURE (m: Model) Domain (): Stores.Domain, NEW

        END;

        Context = POINTER TO ABSTRACT RECORD

            (c: Context) ThisModel (): Model, NEW, ABSTRACT;

            (c: Context) GetSize (OUT w, h: INTEGER), NEW, ABSTRACT;

            (c: Context) SetSize (w, h: INTEGER), NEW, EMPTY;

            (c: Context) Normalize (): BOOLEAN, NEW, ABSTRACT;

            (c: Context) Consider (VAR p: Proposal), NEW, EMPTY;

            (c: Context) MakeVisible (l, t, r, b: INTEGER), NEW, EMPTY

        END;

        Proposal = ABSTRACT RECORD END;

        Message = ABSTRACT RECORD

            model-: Model;

            era-: INTEGER

        END;

        UpdateMsg = EXTENSIBLE RECORD (Message) END;

        NeutralizeMsg = RECORD (Message) END;

    PROCEDURE CopyOf (m: Model): Model;

    PROCEDURE Broadcast (model: Model; VAR msg: Message);

    PROCEDURE Domaincast (d: Stores.Domain; VAR msg: Message);

    PROCEDURE BeginModification (type: INTEGER; m: Model);

    PROCEDURE EndModification (type: INTEGER; m: Model);

    PROCEDURE BeginScript (m: Model; name: Stores.OpName; OUT script: Stores.Operation);

    PROCEDURE EndScript (m: Model; script: Stores.Operation);

    PROCEDURE Do (m: Model; name: Stores.OpName; op: Stores.Operation);

    PROCEDURE LastOp (m: Model): Stores.Operation;

    PROCEDURE Bunch (m: Model);

    PROCEDURE StopBunching (m: Model);

    PROCEDURE Era (m: Model): INTEGER;

END Models.

Figure 1. Model-View-Controller Separation

A *model* is one part of a Model-View-Controller triple. A model represents some data, without knowing how these data may be represented. Representation is performed by a view. There may be several views displaying the same model simultaneously, and possibly in different ways.

After a model has been modified, a *model message* is broadcast in the domain to which this model belongs (e.g., the domain of the document of which the model is a part). Model messages are received by the appropriate views, such that these views can update the display according to the model modification which was performed.

A modification of a model may be permanent, or reversible. To indicate a permanent modification, the procedures *BeginModification/EndModification* must be called before/after the modification(s). Reversible modifications ("undoable" operations) are implemented as *Stores.Operation* objects. Several operations on the same model can be combined into one (i.e., as a whole undoable/redoable) operation with the *BeginScript/EndScript* pair of procedures.

A model may be a container, i.e., contain embedded views. An embedded view can communicate with the model in which it is embedded via a *Context*. A container model provides a context for each embedded view. Using its context, a view can inquire its current size, or it can try to change its size.

All models in a document share the same domain, i.e., their domain fields are either *NIL* (when the document is not displayed in some window) or refer to the same domain. Different displayed documents have different domains.

Examples:

[<u>ObxLines  docu</u>](../../Obx/Docu/Lines.odc.md)

[<u>ObxGraphs  docu</u>](../../Obx/Docu/Graphs.odc.md)

[<u>ObxCaps  docu/sources</u>](../../Obx/Docu/Caps.odc.md)    compound commands (undoable scripts)

CONST **clean**

Possible value for parameter *type* of *BeginModification/EndModification*. Indicates an operation that does not make its document "dirty". Example: modifying a text in a way that is considered as "unimportant", such as collapsing or expanding a text fold (-> StdFolds).

CONST **invisible**

Possible value for parameter *type* of *BeginModification/EndModification*. Indicates an operation that "folds together" with the previous operation, i.e., does not itself become visible in the Undo/Redo menu items. Invisible operations can be used for operations that by themselves may not be expected to appear in an Undo/Redo menu. Example: setting options in a controller. When executing a Redo operation, after a (visible) operation, all invisible operations executed. When executing an Undo operation, first all invisible operations are undone and afterwards the visible operations.

CONST **notUndoable**

Possible value for parameter *type* of *BeginModification/EndModification*. Indicates an operation that cannot be reversed ("undone"). This is important for operations where the undo feature would be too expensive.

TYPE **Model (Stores.Store)**

ABSTRACT

A model represents data, which may be presented by a view.

Models are allocated by specific model directories, e.g., by *Texts.dir*.

Models are used by commands which can operate on the data that is represented by the model.

Models are extended whenever new kinds of displayable data need to be represented.

A model's domain delineates the boundaries of the document in which it is embedded.

A model's *InitDomain* procedure must be implemented if the model may (directly or indirectly) contain other *Stores.Store* objects, e.g., if the model may contain views.

A model's *Internalize*, *Externalize*, and *CopyFrom* procedure must be implemented if the store contains persistent state.

PROCEDURE (m: Model) **Domain** (): Stores.Domain

NEW

Returns the domain of the model, usually representing one document. If the domain is *NIL*, the model is not displayed. The domain is set up by the procedure *Stores.InitDomain*.

TYPE **Context**

ABSTRACT

A context object is part of the model-view recursion scheme of BlackBox. A context is generated and maintained by a container model, and there is one context for every view embedded in the model. The context is carried by the view, so the view can communicate with its context (i.e., with the model in which it is embedded).

A *Context* allows a contained view to communicate with its container.

A *Context* is extended for every container model.

PROCEDURE (c: Context) **ThisModel** (): Model

NEW, ABSTRACT

Returns the context's model. *NIL* may be returned if the context doesn't want to disclose its identity.

*ThisModel* may be narrowed in an extension.

PROCEDURE (c: Context) **GetSize** (OUT w, h: INTEGER)

NEW, ABSTRACT

Returns the width and height of the contained view in its container, in universal units.

Post

w >= 0  &  h >= 0

PROCEDURE (c: Context) **SetSize** (w, h: INTEGER)

NEW, EMPTY

Requests the container to adapt the size of *c*'s view to the given width and height. The container may or may not grant this request.

PROCEDURE (c: Context) **Normalize** (): BOOLEAN

NEW, ABSTRACT

Determines whether the contained view should normalize its persistent state upon externalization, and whether it should not make a modification of this state undoable.

As an example, consider the scroll position of a text view: if the view is in a root context, i.e., in the outermost view level, it should write out position 0 (i.e., "normalized") as its current scroll position upon externalization, and it shouldn't make scroll operations undoable. However, if embedded in a non-root context, it should write out its current scroll position, and should make a scroll operation undoable.

*Normalize* is called in a view's Externalize procedure and when it must be determined whether an operation needs to be undoable or not.

PROCEDURE (c: Context) **Consider** (VAR p: Proposal)

NEW, EMPTY

If an embedded view wants its container to do something, it must ask for such a change by sending the container a *Proposal* via the *Consider* procedure. The container may, but need not, cooperate in an appropriate way. BlackBox currently doesn't predefine proposals of its own.

PROCEDURE (c: Context) **MakeVisible** (l, t, r, b: INTEGER)

NEW, EMPTY

Scroll the container of c's view such that the rectangle *(l, t, r, b)* becomes at least partially visible.

TYPE **Proposal**

ABSTRACT

Base type for all proposals. A proposal is a message which a view can send to the container (via its context object) in which it is embedded.

TYPE **Message**

ABSTRACT

Base type for all model messages.

Messages are used to transmit information from a model to all views which display this model, typically about changes that have occurred in a model.

Messages are extended to describe specific kinds of information to be transmitted.

**model**: Model    model # NIL

The model that has been changed.

**era**-: INTEGER

Used internally.

TYPE **UpdateMsg (Message)**

EXTENSIBLE

All model messages which notify about a model modification must be extensions of *UpdateMsg*. A basic (unextended) *UpdateMsg* indicates that the message's model has changed in some unspecified way.

*UpdateMsgs* are used to notify all views displaying a given model about a change of this model.

*UpdateMsgs* are extended in order to update the display in more specific ways than to redraw the whole view (i.e., faster and with less screen flicker), i.e., to allow partial updates.

TYPE **NeutralizeMsg (Message)**

Extension

This message is sent by the framework to indicate that marks (selection marks and the like) should be removed.

PROCEDURE **CopyOf** (m: Model): Model

Returns a (deep) copy of the model. Internally, it clones *m* and calls its *CopyFrom* method.

Pre

m # NIL    20

Post

result # NIL

PROCEDURE **Broadcast** (model: Model; VAR msg: Message)

Broadcast *msg* for *model*. Before broadcasting, parameter *model* is assigned to the message's *model*-field. The broadcast is actually performed only if *model.domain # NIL*.

*Broadcast* is called by models whenever models need to transmit information to the views by which they are displayed. In contrast to *Domaincast*, *Broadcast* sends *msg* only to the views which have *model* as their model.

The handler of a model message may not recursively broadcast another model message.

Pre

model # NIL    20

no recursion    21

Post

msg.model = model

PROCEDURE **Domaincast** (d: Stores.Domain; VAR msg: Message)

This procedure sends a message to a particular domain, or does nothing if the domain is *NIL*. Every view in the domain receives the message. Normally, you'll use *Broadcast* to notify only the views on a particular model. *Domaincast* is only necessary if the contents of one view (e.g., a text ruler) influences something outside of the view itself (e.g., the formatting of the text below the ruler). The domaincast is actually performed only if *d # NIL*.

A domaincast may not be performed if another one is still in progress for this domain (no recursion).

Pre

no recursion    20

Post

msg.model = NIL

PROCEDURE **BeginModification** (type: INTEGER; m: Model)

If model *m* is modified in a way that cannot be undone, the modification(s) must be bracketed by calls to *BeginModification* and *EndModification* with parameter *type* set to *notUndoable*.

Pre

m # NIL    20

type IN {clean, notUndoable, invisible}    21

PROCEDURE **EndModification** (type: INTEGER; m: Model)

If model *m* is modified in a way that cannot be undone, the modification(s) must be bracketed by calls to *BeginModification* and *EndModification* with parameter *type* set to *notUndoable*.

Pre

m # NIL    20

type IN {clean, notUndoable, invisible}    21

PROCEDURE **BeginScript** (m: Model; name: Stores.OpName; OUT script: Stores.Operation)

To make a sequence of undoable operations undoable as a whole, the sequence should be bracketed by calls to *BeginScript* and *EndScript*.

Pre

m # NIL    20

Post

script # NIL

PROCEDURE **EndScript** (m: Model; script: Stores.Operation)

To make a sequence of undoable operations undoable as a whole, the sequence should be bracketed by calls to *BeginScript* and *EndScript*. The same script which has been returned in *BeginScript* must be passed to *EndScript*.

Pre

m # NIL    20

script # NIL    21

PROCEDURE **Do** (m: Model; name: Stores.OpName; op: Stores.Operation)

This procedure is called to execute an operation on a model. The operation's *Do* procedure is called, and the operation is recorded for a later undo.

Pre

m # NIL    20

op # NIL    21

Post

op.inUse

PROCEDURE **LastOp** (m: Model): Stores.Operation

This procedure returns the most recently executed operation on the given model. It can be used to decide whether to bunch several successive operations into one single atomic operation, e.g., if a character is typed into a text, it may be bunched with the previously inserted character; such that an undo operates on the whole text typed in, and not on one character per undo.

Pre

m # NIL    20

PROCEDURE **Bunch** (m: Model)

Notify model that another action was bunched to the most recently executed operation. After it has been determined that the new operation can be merged into the most recent operation (using *LastOp* for the test), the previous operation can be modified (e.g., a character appended to its string of characters that were typed in) and this modification made known to the framework by calling *Bunch*.

Pre

m # NIL    20

PROCEDURE **StopBunching **(m: Model)

Prevents any further bunching on this model's current operation.

Pre

m # NIL    20

PROCEDURE **Era** (m: Model): INTEGER

Called internally.

Pre

m # NIL    20

