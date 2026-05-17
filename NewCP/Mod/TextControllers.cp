MODULE TextControllers;
(*
   First slice of the BlackBox `TextControllers` port.

   `Controller` is the abstract mediator between a `TextViews.View`
   (the visible editor pane) and its underlying `TextModels.Model`
   text storage.  It owns the caret position, the selection range,
   and the input filter chain — concrete implementations (`StdCtrl`
   in BB) handle keystroke processing, mouse tracking, paste
   filtering, and so on.

   This slice carries:

     - All the BB CONSTs the higher-level framework references
       (`noAutoScroll`, `noAutoIndent`, `none`, key strings, version
       gates).
     - The abstract surface (`Controller`, `Directory`) plus the
       inline `view-` / `text-` fields so importers can type-route
       messages against them.
     - The five public message records (`FilterPref`,
       `FilterPollCursorMsg`, `FilterTrackMsg`, `SetCaretMsg`,
       `SetSelectionMsg`) plus the abstract `ModelMessage` base.
     - Abstract method declarations on Controller (`CaretPos`,
       `SetCaret`, `GetSelection`, `SetSelection`) with deferred
       bodies — the concrete `StdCtrl` implementation lands in a
       follow-up slice.
     - Concrete EXTENSIBLE-on-Controller methods (`Internalize2`,
       `Externalize2`, `InitView2`, `ThisView`) that mirror BB's
       wire-format and view-binding behaviour faithfully.
     - The `dir-, stdDir-` module variables and the `SetDir`,
       `Install`, `Focus`, `SetCaret`, `SetSelection` procedures
       that drive controller dispatch from outside.

   The concrete `StdCtrl` body — caret tracking, mouse / keyboard
   handling, selection management, undo integration — is intentionally
   not in this slice; landing it requires the full StdView slice for
   `TextViews` to exist first.
*)

IMPORT
    Stores, Models, Views, Controllers, Properties, Containers,
    TextModels, TextRulers, TextSetters, TextViews;

CONST
    (** View options the Container framework consults (see
        BB Containers.View): suppress auto-scroll on selection
        change and auto-indent on RETURN. *)
    noAutoScroll* = 16;
    noAutoIndent* = 17;

    (** Sentinel for SetCaret / SetSelection: -1 = "no position"
        / "no selection".  BB-faithful. *)
    none* = -1;

    (* Track mode used internally by StdCtrl (later slice). *)
    chars = 0; words = 1; lines = 2;

    (* Special key codepoints the StdCtrl input filter recognises. *)
    enter = 03X; rdel = 07X; ldel = 08X;

    aL = 01CX; aR = 01DX; aU = 01EX; aD = 01FX;
    pL = 010X; pR = 011X; pU = 012X; pD = 013X;
    dL = 014X; dR = 015X; dU = 016X; dD = 017X;

    viewcode = TextModels.viewcode;
    tab      = TextModels.tab;
    line     = TextModels.line;
    para     = TextModels.para;

    boundCaret = TRUE;

    (** Max run length StdCtrl will inspect when fetching the
        attribute span around the caret — keeps "what fonts are
        selected" cheap on huge selections. *)
    lenCutoff = 2000;

    (* Property-message routing keys used by the BB framework. *)
    attrChangeKey* = "#Text:AttributeChange";
    resizingKey*   = "#System:Resizing";
    insertingKey*  = "#System:Inserting";
    deletingKey*   = "#System:Deleting";
    movingKey*     = "#System:Moving";
    copyingKey*    = "#System:Copying";
    linkingKey*    = "#System:Linking";
    replacingKey*  = "#System:Replacing";

    minVersion    = 0;
    maxVersion    = 0;
    maxStdVersion = 0;

TYPE
    (** Abstract container-controller for a text view.  `view-` is
        the visible pane this controller mediates; `text-` is the
        underlying text storage (in BB this is always
        `view.ThisText()` when `view # NIL`, so the field is
        redundant cache rather than independent state). *)
    ControllerDesc* = ABSTRACT RECORD (Containers.ControllerDesc)
        view-: TextViews.View;
        text-: TextModels.Model
    END;
    Controller*     = POINTER TO ControllerDesc;

    (** Abstract directory — the framework factory that builds a
        fresh controller for a given option set.  Concrete
        `StdDirectory` (deferred to later slice) supplies
        `NewController`. *)
    DirectoryDesc* = ABSTRACT RECORD (Containers.DirectoryDesc) END;
    Directory*     = POINTER TO DirectoryDesc;

    (** Paste-filter property: ask the framework whether a given
        cursor location should accept a paste of `controller`'s
        current selection.  Filter chain sets `filter` to TRUE if
        any handler vetoes the paste. *)
    FilterPref* = RECORD (Properties.Preference)
        controller*: Controller;
        frame*:      Views.Frame;
        x*, y*:      INTEGER;
        filter*:     BOOLEAN
    END;

    (** Cursor-shape filter message.  Frameworks watching for it
        can override the cursor at a particular pixel location;
        `done` lets a handler short-circuit the rest of the chain. *)
    FilterPollCursorMsg* = RECORD (Controllers.Message)
        controller*: Controller;
        x*, y*:      INTEGER;
        cursor*:     INTEGER;
        done*:       BOOLEAN
    END;

    (** Drag-tracking filter message — same flow as
        `FilterPollCursorMsg` but for mouse-drag tracking. *)
    FilterTrackMsg* = RECORD (Controllers.Message)
        controller*: Controller;
        x*, y*:      INTEGER;
        modifiers*:  SET;
        done*:       BOOLEAN
    END;

    (** Base for `SetCaretMsg`/`SetSelectionMsg` so virtual model
        extensions (e.g. mark layers) can hook the same broadcast
        the framework uses to drive caret / selection updates. *)
    ModelMessage* = ABSTRACT RECORD (Models.Message) END;

    (** Broadcast: move the caret to `pos`.  Sent by
        `TextControllers.SetCaret` (the module-level proc) so
        every controller bound to the model gets a chance to
        respond. *)
    SetCaretMsg* = EXTENSIBLE RECORD (ModelMessage)
        pos*: INTEGER
    END;

    (** Broadcast: select the range `[beg, end)`.  Same chain as
        `SetCaretMsg`. *)
    SetSelectionMsg* = EXTENSIBLE RECORD (ModelMessage)
        beg*, end*: INTEGER
    END;

    (** Concrete-minimum controller body.  Carries the caret
        position and selection range as plain integer state and
        supplies the four abstract methods (`CaretPos`,
        `SetCaret`, `GetSelection`, `SetSelection`) with direct
        field-update bodies.

        BB's `StdCtrl` is far larger — it tracks track-mode,
        cached reader/writer, auto-scroll bounds, blink ticks,
        selection pin points, modifier state, and a complete
        keystroke / mouse / paste filter chain.  This slice is a
        BB-faithful prefix: the fields' MEANING matches BB
        (`carPos = none` when no caret, `selBeg = selEnd` when
        no selection, both clamped to `text.Length()`), so a
        later slice can grow the record without breaking
        anything callers depend on today.  The non-prefix BB
        fields (cachedRd, cachedWr, autoBeg/autoEnd, carLast,
        carX/lastX, carTick, carVisible, aliasSel*, selPin*,
        lastStep) intentionally aren't here — they're worthless
        without input handling, which needs the StdView slice. *)
    StdCtrlDesc = RECORD (ControllerDesc)
        carPos: INTEGER;
        selBeg, selEnd: INTEGER
    END;
    StdCtrl = POINTER TO StdCtrlDesc;

    (** Concrete-minimum directory: `NewController` allocates a
        fresh `StdCtrl` with no caret / no selection.  BB's
        StdDirectory is also empty-record — the option-set
        plumbing it gets via `NewController(opts)` is delegated
        to the controller via inherited fields (which we don't
        carry yet).  Adding `opts` support is a follow-up. *)
    StdDirectoryDesc = RECORD (DirectoryDesc) END;
    StdDirectory     = POINTER TO StdDirectoryDesc;

VAR
    (** Active controller-directory.  `SetDir` overrides; `stdDir`
        is the framework default and never gets replaced. *)
    dir-, stdDir-: Directory;
    (* Module-private storage of the StdDirectory instance so the
       body can NEW it through its concrete type before exposing
       it via `dir-`/`stdDir-` (which are typed as the abstract
       Directory and can't take a NEW). *)
    std: StdDirectory;

(* ─── Controller surface ───────────────────────────────────────
   `Internalize2`, `Externalize2`, `InitView2`, `ThisView` are
   concrete EXTENSIBLE — they mirror BB's bodies faithfully so
   wire-format and view-binding work as soon as the slice loads.
   The caret / selection methods (`CaretPos`, `SetCaret`,
   `GetSelection`, `SetSelection`) are NEW + ABSTRACT — concrete
   `StdCtrl` supplies them later.
*)

PROCEDURE (c: Controller) Internalize2- (VAR rd: Stores.Reader), NEW, EXTENSIBLE;
    VAR v: INTEGER;
BEGIN
    rd.ReadVersion(minVersion, maxVersion, v)
END Internalize2;

PROCEDURE (c: Controller) Externalize2- (VAR wr: Stores.Writer), NEW, EXTENSIBLE;
BEGIN
    wr.WriteVersion(maxVersion)
END Externalize2;

PROCEDURE (c: Controller) InitView2* (v: Views.View), NEW, EXTENSIBLE;
    VAR m: Models.Model;
BEGIN
    ASSERT((v = NIL) # (c.view = NIL), 21);
    IF c.view = NIL THEN ASSERT(v IS TextViews.View, 22) END;
    IF v # NIL THEN
        c.view := v(TextViews.View);
        m := c.view.ThisModel();
        IF m # NIL THEN
            (* TextViews.View.ThisModel() returns the underlying
               TextModels.Model widened through Containers.Model;
               narrow it back here so c.text carries the concrete
               text-model interface its callers expect. *)
            c.text := m(TextModels.Model)
        ELSE
            c.text := NIL
        END
    ELSE
        c.view := NIL;
        c.text := NIL
    END
END InitView2;

PROCEDURE (c: Controller) ThisView* (): TextViews.View, NEW, EXTENSIBLE;
BEGIN
    RETURN c.view
END ThisView;

(** Caret position (or `none`).  Concrete in StdCtrl.
    Overrides Containers.Controller.CaretPos (base returns -1 = none). *)
PROCEDURE (c: Controller) CaretPos* (): INTEGER, ABSTRACT;

(** Move the caret to `pos` (or hide if `pos = none`).
    pre: pos = none  OR  0 <= pos <= c.text.Length() *)
PROCEDURE (c: Controller) SetCaret* (pos: INTEGER), NEW, ABSTRACT;

(** Read the selection range; empty selection signaled by beg = end.
    post: beg = end  OR  0 <= beg <= end <= c.text.Length()
    Overrides Containers.Controller.GetSelection (base returns -1,-1). *)
PROCEDURE (c: Controller) GetSelection* (OUT beg, end: INTEGER), ABSTRACT;

(** Set the selection range; empty selection signaled by beg = end.
    pre: beg = end  OR  0 <= beg < end <= c.text.Length() *)
PROCEDURE (c: Controller) SetSelection* (beg, end: INTEGER), NEW, ABSTRACT;

(** TRUE iff `c` has a non-empty selection.  Concrete EXTENSIBLE
    default: calls GetSelection and tests `beg # end`.  BB-faithful.
    Used by every reader-side BB module (In, ETHConv, search, etc.)
    to decide whether to pull from the selection or the whole text. *)
PROCEDURE (c: Controller) HasSelection* (): BOOLEAN, NEW, EXTENSIBLE;
    VAR beg, end: INTEGER;
BEGIN
    c.GetSelection(beg, end);
    RETURN beg # end
END HasSelection;

(* ─── StdCtrl concrete bodies ──────────────────────────────────
   Field-update implementations of the four abstract methods.
   Preconditions match BB and Controller's contract; callers
   that violate them get ASSERT-driven traps so misuse is loud.
*)

PROCEDURE (c: StdCtrl) CaretPos* (): INTEGER;
BEGIN
    RETURN c.carPos
END CaretPos;

PROCEDURE (c: StdCtrl) SetCaret* (pos: INTEGER);
BEGIN
    (* BB precondition: `pos = none OR (0 <= pos <= text.Length())`.
       The model-length check is skipped in this slice (c.text is
       only populated after InitView2 binds a view); the module-
       level TextControllers.SetCaret already asserts against the
       model on the broadcast path. *)
    ASSERT((pos = none) OR (pos >= 0), 20);
    c.carPos := pos
END SetCaret;

PROCEDURE (c: StdCtrl) GetSelection* (OUT beg, end: INTEGER);
BEGIN
    beg := c.selBeg;
    end := c.selEnd
END GetSelection;

PROCEDURE (c: StdCtrl) SetSelection* (beg, end: INTEGER);
BEGIN
    (* BB: `beg = end OR (0 <= beg < end <= text.Length())`.
       Length check deferred for the same reason as SetCaret. *)
    IF beg # end THEN
        ASSERT(0 <= beg, 20);
        ASSERT(beg < end, 21)
    END;
    c.selBeg := beg;
    c.selEnd := end
END SetSelection;

(* ─── StdCtrl keyboard input ───────────────────────────────────
   HandleKey processes typed characters and editing keys.  The model
   must be a TextModels.Doc (our concrete editable type); if the
   controller's `text` field is NIL or is not a Doc, the call is
   a no-op.

   Special codepoints handled (matching BB StdCtrl conventions):
     ldel  (08X) — delete char before caret (backspace)
     rdel  (07X) — delete char after caret (forward delete)
     line  (0DX) — insert line separator
     para  (0EX) — insert paragraph separator
     Any other printable CHAR — insert at caret
*)

PROCEDURE (c: StdCtrl) HandleKey* (ch: CHAR), NEW;
    VAR doc:  TextModels.Doc;
        beg, end, pos: INTEGER;
BEGIN
    IF (c.text = NIL) OR ~(c.text IS TextModels.Doc) THEN RETURN END;
    doc := c.text(TextModels.Doc);

    (* If there is a selection, delete it first for any editing op. *)
    beg := c.selBeg; end := c.selEnd;
    IF beg # end THEN
        IF beg > end THEN pos := beg; beg := end; end := pos END;
        doc.DeleteRange(beg, end);
        c.carPos := beg;
        c.selBeg := beg; c.selEnd := beg
    END;

    pos := c.carPos;
    IF pos = none THEN pos := 0 END;
    IF pos < 0 THEN pos := 0 END;
    IF pos > doc.len THEN pos := doc.len END;

    IF ch = ldel THEN
        (* Backspace: delete the character before the caret. *)
        IF pos > 0 THEN
            doc.DeleteRange(pos - 1, pos);
            DEC(pos)
        END
    ELSIF ch = rdel THEN
        (* Forward delete: delete the character at the caret. *)
        IF pos < doc.len THEN
            doc.DeleteRange(pos, pos + 1)
        END
    ELSIF ch >= ' ' THEN
        (* Printable character: insert at caret. *)
        doc.InsertChar(pos, ch);
        INC(pos)
    ELSIF (ch = line) OR (ch = para) THEN
        (* Line or paragraph separator: insert as-is. *)
        doc.InsertChar(pos, ch);
        INC(pos)
    END;
    c.carPos := pos
END HandleKey;


(* ─── Module-level HandleKey dispatch ─────────────────────────
   Convenience: type a character into the focused StdCtrl.
   Returns FALSE if no focused StdCtrl is available. *)
PROCEDURE HandleKey* (ch: CHAR): BOOLEAN;
    VAR c: Controller;
BEGIN
    c := Focus();
    IF (c # NIL) & (c IS StdCtrl) THEN
        c(StdCtrl).HandleKey(ch);
        RETURN TRUE
    END;
    RETURN FALSE
END HandleKey;


(* ─── Directory surface ────────────────────────────────────────
   `NewController(opts)` builds a fresh controller carrying the
   given option mask; `New()` is the convenience overload for
   empty-options.  StdDirectory.NewController allocates a
   StdCtrl with neutral caret/selection state.
*)

PROCEDURE (d: Directory) NewController* (opts: SET): Controller, NEW, ABSTRACT;

PROCEDURE (d: Directory) New* (): Controller, NEW, EXTENSIBLE;
BEGIN
    RETURN d.NewController({})
END New;

PROCEDURE (d: StdDirectoryDesc) NewController* (opts: SET): Controller;
    VAR c: StdCtrl;
BEGIN
    NEW(c);
    c.carPos := none;
    c.selBeg := 0;
    c.selEnd := 0;
    RETURN c
END NewController;

(* ─── Module-level procedures ─────────────────────────────────
   `SetDir` / `Install` are the host-side installation hooks;
   `Focus`, `SetCaret`, `SetSelection` are the public entry points
   external code uses to drive the controller chain. *)

PROCEDURE SetDir* (d: Directory);
BEGIN
    ASSERT(d # NIL, 20);
    dir := d
END SetDir;

PROCEDURE Install*;
BEGIN
    TextViews.SetCtrlDir(dir)
END Install;

PROCEDURE Focus* (): Controller;
    VAR v: Views.View; c: Containers.Controller;
BEGIN
    v := Controllers.FocusView();
    IF (v # NIL) & (v IS TextViews.View) THEN
        c := v(TextViews.View).controller;
        IF (c # NIL) & (c IS Controller) THEN
            RETURN c(Controller)
        ELSE
            RETURN NIL
        END
    ELSE
        RETURN NIL
    END
END Focus;

PROCEDURE SetCaret* (text: TextModels.Model; pos: INTEGER);
(** pre: text # NIL,  pos = none  OR  0 <= pos <= text.Length() *)
    VAR cm: SetCaretMsg;
BEGIN
    ASSERT(text # NIL, 20);
    ASSERT(none <= pos, 21);
    ASSERT(pos <= text.Length(), 22);
    cm.pos := pos;
    Models.Broadcast(text, cm)
END SetCaret;

PROCEDURE SetSelection* (text: TextModels.Model; beg, end: INTEGER);
(** pre: text # NIL,  beg = end  OR  0 <= beg < end <= text.Length() *)
    VAR sm: SetSelectionMsg;
BEGIN
    ASSERT(text # NIL, 20);
    IF beg # end THEN
        ASSERT(0 <= beg, 21);
        ASSERT(beg < end, 22);
        ASSERT(end <= text.Length(), 23)
    END;
    sm.beg := beg;
    sm.end := end;
    Models.Broadcast(text, sm)
END SetSelection;

BEGIN
    (* Install StdDirectory as both the framework default and the
       currently-active directory.  BB does this in StdInterpreter
       at boot via an explicit SetDir call; doing it in the body
       keeps the "import TextControllers and it works" property. *)
    NEW(std);
    stdDir := std;
    dir := std
END TextControllers.
