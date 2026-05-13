MODULE Mechanisms;
(*
   First slice of the BlackBox `Mechanisms` port.

   `Mechanisms` is a thin trampoline module: every public
   procedure delegates straight into a `hook` module-variable.
   Each hook method is ABSTRACT on the base, so the host UI
   installs a concrete subclass via `SetHook` once at startup
   and from then on every framework caller hits the host
   implementation through the trampoline.

   The trampoline pattern decouples three layers cleanly:
   - the framework (`Containers`, `TextControllers`,
     `FormControllers`) calls `Mechanisms.TrackToResize`,
     `MarkFocusBorder`, `PopUpAndSelect`, etc. — without
     importing any host-UI type;
   - the host (e.g. `WinMechanisms` on Windows, `MacMechanisms`
     on Mac) extends `Hook` with a concrete subclass and
     installs it at startup;
   - this module is the abstract-base + indirection that
     turns "I want to track a resize" into "host, please do
     a resize-track loop".

   This slice ships the full BlackBox surface — the abstract
   hook record, the SetHook installer, and every trampoline.
   No host concrete subclass yet (that's a UI-layer concern,
   slated for the iGui host backend).

   Deferred: nothing — this module is small enough to land
   whole.  The hook surface is pure data (signatures only);
   the only "work" is the trampoline forwarding.
*)

    IMPORT Kernel, Views;

    CONST
        (** FocusBorderCursor / SelBorderCursor result sentinels.
            Anything outside [inside, outside) is a defined
            Ports cursor constant. *)
        inside*  = -1;
        outside* = -2;

        (** TrackToResize.op outcome. *)
        cancelResize* = 0;
        resize*       = 1;

        (** TrackToDrop.op outcome. *)
        cancelDrop* = 0;
        copy*       = 1;
        move*       = 2;
        link*       = 3;

        (** TrackToPick.op outcome. *)
        cancelPick*  = 0;
        pick*        = 1;
        pickForeign* = 2;


    TYPE
        (** Abstract Mechanisms hook.  Extends `Kernel.Hook` so
            the runtime's hook registry can store us alongside
            the other framework hooks. *)
        HookDesc* = ABSTRACT RECORD (Kernel.HookDesc) END;
        Hook*     = POINTER TO HookDesc;


    VAR
        (** The currently-installed hook.  Set once at host
            startup via `SetHook`; trampolines below all
            de-reference this. *)
        hook: Hook;


    (* -- Hook installation ----------------------------------------------- *)

    PROCEDURE SetHook* (h: Hook);
    BEGIN
        hook := h
    END SetHook;


    (* -- ABSTRACT method declarations ------------------------------------ *)

    PROCEDURE (hk: Hook) MarkFocusBorder*
        (host: Views.Frame; focus: Views.View;
         l, t, r, b: INTEGER; show: BOOLEAN), NEW, ABSTRACT;

    PROCEDURE (hk: Hook) MarkSingletonBorder*
        (host: Views.Frame; view: Views.View;
         l, t, r, b: INTEGER; show: BOOLEAN), NEW, ABSTRACT;

    PROCEDURE (hk: Hook) FocusBorderCursor*
        (host: Views.Frame; view: Views.View;
         l, t, r, b: INTEGER;
         x, y: INTEGER): INTEGER, NEW, ABSTRACT;

    PROCEDURE (hk: Hook) SelBorderCursor*
        (host: Views.Frame; view: Views.View;
         l, t, r, b: INTEGER;
         x, y: INTEGER): INTEGER, NEW, ABSTRACT;

    PROCEDURE (hk: Hook) TrackToResize*
        (host: Views.Frame; view: Views.View;
         minW, maxW, minH, maxH: INTEGER;
         VAR l, t, r, b: INTEGER;
         VAR op: INTEGER;
         VAR x, y: INTEGER;
         VAR buttons: SET), NEW, ABSTRACT;

    PROCEDURE (hk: Hook) TrackToDrop*
        (source: Views.Frame; view: Views.View;
         isSingle: BOOLEAN; w, h, rx, ry: INTEGER;
         VAR dest: Views.Frame; VAR destX, destY: INTEGER;
         VAR op: INTEGER;
         VAR x, y: INTEGER;
         VAR buttons: SET), NEW, ABSTRACT;

    PROCEDURE (hk: Hook) TrackToPick*
        (source: Views.Frame;
         VAR dest: Views.Frame; VAR destX, destY: INTEGER;
         VAR op: INTEGER;
         VAR x, y: INTEGER;
         VAR buttons: SET), NEW, ABSTRACT;

    PROCEDURE (hk: Hook) PopUpAndSelect*
        (f: Views.Frame;
         n, this: INTEGER;
         s: ARRAY OF ARRAY OF CHAR;
         enabled, checked: ARRAY OF BOOLEAN;
         VAR i: INTEGER;
         VAR x, y: INTEGER;
         VAR buttons: SET), NEW, ABSTRACT;


    (* -- Trampolines ---------------------------------------------------- *)

    PROCEDURE MarkFocusBorder*
        (host: Views.Frame; focus: Views.View;
         l, t, r, b: INTEGER; show: BOOLEAN);
    BEGIN
        hook.MarkFocusBorder(host, focus, l, t, r, b, show)
    END MarkFocusBorder;

    PROCEDURE MarkSingletonBorder*
        (host: Views.Frame; view: Views.View;
         l, t, r, b: INTEGER; show: BOOLEAN);
    BEGIN
        hook.MarkSingletonBorder(host, view, l, t, r, b, show)
    END MarkSingletonBorder;

    PROCEDURE FocusBorderCursor*
        (host: Views.Frame; view: Views.View;
         l, t, r, b: INTEGER;
         x, y: INTEGER): INTEGER;
    BEGIN
        RETURN hook.FocusBorderCursor(host, view, l, t, r, b, x, y)
    END FocusBorderCursor;

    PROCEDURE SelBorderCursor*
        (host: Views.Frame; view: Views.View;
         l, t, r, b: INTEGER;
         x, y: INTEGER): INTEGER;
    BEGIN
        RETURN hook.SelBorderCursor(host, view, l, t, r, b, x, y)
    END SelBorderCursor;

    PROCEDURE TrackToResize*
        (host: Views.Frame; view: Views.View;
         minW, maxW, minH, maxH: INTEGER;
         VAR l, t, r, b: INTEGER;
         VAR op: INTEGER;
         VAR x, y: INTEGER;
         VAR buttons: SET);
    BEGIN
        hook.TrackToResize(host, view, minW, maxW, minH, maxH,
                           l, t, r, b, op, x, y, buttons)
    END TrackToResize;

    PROCEDURE TrackToDrop*
        (source: Views.Frame; view: Views.View;
         isSingle: BOOLEAN; w, h, rx, ry: INTEGER;
         VAR dest: Views.Frame; VAR destX, destY: INTEGER;
         VAR op: INTEGER;
         VAR x, y: INTEGER;
         VAR buttons: SET);
    BEGIN
        hook.TrackToDrop(source, view, isSingle, w, h, rx, ry,
                         dest, destX, destY, op, x, y, buttons)
    END TrackToDrop;

    PROCEDURE TrackToPick*
        (source: Views.Frame;
         VAR dest: Views.Frame; VAR destX, destY: INTEGER;
         VAR op: INTEGER;
         VAR x, y: INTEGER;
         VAR buttons: SET);
    BEGIN
        hook.TrackToPick(source, dest, destX, destY,
                         op, x, y, buttons)
    END TrackToPick;

    PROCEDURE PopUpAndSelect*
        (f: Views.Frame;
         n, this: INTEGER;
         s: ARRAY OF ARRAY OF CHAR;
         enabled, checked: ARRAY OF BOOLEAN;
         VAR i: INTEGER;
         VAR x, y: INTEGER;
         VAR buttons: SET);
    BEGIN
        hook.PopUpAndSelect(f, n, this, s, enabled, checked,
                            i, x, y, buttons)
    END PopUpAndSelect;


    (** Lets a host's startup probe whether a hook is already
        installed.  TRUE = ready to dispatch; FALSE = calling
        any trampoline would NIL-deref. *)
    PROCEDURE HookIsInstalled* (): BOOLEAN;
    BEGIN
        RETURN hook # NIL
    END HookIsInstalled;


END Mechanisms.
