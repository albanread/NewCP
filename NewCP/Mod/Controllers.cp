MODULE Controllers;
(*
   First slice of the BlackBox `Controllers` port.

   `Controllers` is the input-routing / focus-routing layer that
   sits between `Views` and the host UI.  The full module
   (~720 lines) is two layered concerns:

   1. A pile of MESSAGE records used by every controller-flavoured
      View to receive scrolling / paging / cursor / drag-drop /
      edit / paste / poll-state events.  These are pure data —
      no methods — but extending them is how Containers and the
      concrete editor modules (TextControllers / FormControllers)
      hook into the framework.

   2. A "forwarder list" + path-tracking machinery that the host
      uses to dispatch broadcasts.  That half depends on
      Kernel.TrapCleaner and Services.Action, neither of which is
      ported yet; it also has zero shape impact on the
      framework's static type identity.

   This slice ships (1) plus the `Controller` ABSTRACT base type
   itself.  Concrete container-flavoured views — `Containers.View`
   currently references `Stores.Store` for `alienCtrl`, but a
   future slice would tighten that to `Controllers.Controller`
   once `Containers.View.controller` lands.

   Deferred (called out below): `Forwarder`, `path` module var,
   `TrapCleaner`, `BalanceCheckAction`, `WaitAction`, and every
   module-level procedure (`InitForwarder`, `BroadcastMessage`,
   `PassFocus`, …).  The runtime input plumbing those implement
   isn't reachable until we have a host UI surface.
*)

    IMPORT Stores, Views;

    CONST
        (** ForwardTarget — controller-list traversal mode. *)
        targetPath* = TRUE;
        frontPath*  = FALSE;

        (** ScrollMsg.op *)
        decLine* = 0;
        incLine* = 1;
        decPage* = 2;
        incPage* = 3;
        gotoPos* = 4;

        (** PageMsg.op *)
        nextPageX* = 0;
        nextPageY* = 1;
        gotoPageX* = 2;
        gotoPageY* = 3;

        (** PollOpsMsg.valid / EditMsg.op *)
        cut*       = 0;
        copy*      = 1;
        pasteChar* = 2;
        paste*     = 4;

        (** TrackMsg.modifiers / EditMsg.modifiers *)
        doubleClick* = 0;    (** clicking history *)
        extend*      = 1;
        modify*      = 2;    (** modifier keys *)

        (** PollDropMsg.mark / PollDrop mark *)
        noMark* = FALSE;
        mark*   = TRUE;

        (** PollDropMsg.show / PollDrop show *)
        hide* = FALSE;
        show* = TRUE;

        minVersion = 0;
        maxVersion = 0;


    TYPE
        (** Re-export the Views-side controller-message base.  All
            controller messages below extend this; subclassing the
            framework's `Views.CtrlMessage` directly works just as
            well, but the alias matches the BlackBox name and lets
            client code spell `Controllers.Message`. *)
        Message* = Views.CtrlMessage;

        (** Sent to ask "who's the current focus frame?"  EXTENSIBLE
            because Containers.PollFocusMsg adds an `all` flag. *)
        PollFocusMsg* = EXTENSIBLE RECORD (Message)
            focus*: Views.Frame          (** OUT, preset to NIL *)
        END;

        (** Queries the View's scrollable section: which dimension,
            total/visible sizes, scroll position, validity. *)
        PollSectionMsg* = RECORD (Message)
            focus*, vertical*: BOOLEAN;  (** IN *)
            wholeSize*:        INTEGER;  (** OUT, preset to 1 *)
            partSize*:         INTEGER;  (** OUT, preset to 1 *)
            partPos*:          INTEGER;  (** OUT, preset to 0 *)
            valid*, done*:     BOOLEAN   (** OUT, preset (FALSE, FALSE) *)
        END;

        (** Probes the View's selection / paste operations. *)
        PollOpsMsg* = RECORD (Message)
            type*:       Stores.TypeName;  (** OUT, preset "" *)
            pasteType*:  Stores.TypeName;  (** OUT, preset "" *)
            singleton*:  Views.View;       (** OUT, preset NIL *)
            selectable*: BOOLEAN;          (** OUT, preset FALSE *)
            valid*:      SET               (** OUT, preset {} *)
        END;

        (** Scroll request — direction + op + position. *)
        ScrollMsg* = RECORD (Message)
            focus*, vertical*: BOOLEAN;    (** IN *)
            op*:               INTEGER;    (** IN *)
            pos*:              INTEGER;    (** IN *)
            done*:             BOOLEAN     (** OUT, preset FALSE *)
        END;

        (** Page navigation request. *)
        PageMsg* = RECORD (Message)
            op*:               INTEGER;    (** IN *)
            pageX*, pageY*:    INTEGER;    (** IN *)
            done*, eox*, eoy*: BOOLEAN     (** OUT, preset (FALSE,FALSE,FALSE) *)
        END;

        (** Periodic tick for blink / animation. *)
        TickMsg* = RECORD (Message)
            tick*: INTEGER                  (** IN *)
        END;

        (** Marking request — show/hide focus marks. *)
        MarkMsg* = RECORD (Message)
            show*:  BOOLEAN;                (** IN *)
            focus*: BOOLEAN                 (** IN *)
        END;

        (** Selection request — select/deselect all. *)
        SelectMsg* = RECORD (Message)
            set*: BOOLEAN                   (** IN *)
        END;

        (** Base for messages that request focus before being
            handled.  Subclasses set `requestFocus := TRUE` if the
            framework should retarget focus to the message's home
            frame before the message is dispatched. *)
        RequestMessage* = ABSTRACT RECORD (Message)
            requestFocus*: BOOLEAN          (** OUT, preset FALSE *)
        END;

        (** Edit op — cut / copy / paste / paste-character. *)
        EditMsg* = RECORD (RequestMessage)
            op*:        INTEGER;            (** IN *)
            modifiers*: SET;                (** IN, op IN {pasteChar} *)
            char*:      CHAR;               (** IN, op = pasteChar *)
            view*:      Views.View;         (** IN, op = paste *)
            w*, h*:     INTEGER;            (** IN, op = paste; OUT, op IN {cut,copy} *)
            isSingle*:  BOOLEAN;            (** OUT, op IN {cut,copy} *)
            clipboard*: BOOLEAN             (** IN, op IN {cut,copy,paste} *)
        END;

        (** Swap an embedded view in-place. *)
        ReplaceViewMsg* = RECORD (RequestMessage)
            old*, new*: Views.View          (** IN *)
        END;

        (** Base for cursor-coordinate messages — every concrete
            cursor event carries (x, y) in the home-frame's user
            coords. *)
        CursorMessage* = ABSTRACT RECORD (RequestMessage)
            x*, y*: INTEGER                 (** IN; translate when forwarding *)
        END;

        (** Cursor-shape probe — set `cursor` to the appropriate
            Ports.<cursor> constant for the hover position. *)
        PollCursorMsg* = RECORD (CursorMessage)
            cursor*:    INTEGER;            (** OUT, preset to Ports.arrowCursor *)
            modifiers*: SET                 (** IN *)
        END;

        (** Track/drag cursor event. *)
        TrackMsg* = RECORD (CursorMessage)
            modifiers*: SET                 (** IN *)
        END;

        (** Mouse-wheel rotation event. *)
        WheelMsg* = RECORD (CursorMessage)
            done*:               BOOLEAN;   (** OUT, set if handled *)
            op*, nofLines*:      INTEGER
        END;

        (** Base for drag/drop transfer messages. *)
        TransferMessage* = ABSTRACT RECORD (CursorMessage)
            source*:             Views.Frame; (** IN, home frame of originator *)
            sourceX*, sourceY*:  INTEGER       (** IN, reference point *)
        END;

        (** "Would you accept a drop here?" — receiver sets `dest`
            iff the target frame can host the view. *)
        PollDropMsg* = RECORD (TransferMessage)
            mark*:     BOOLEAN;             (** IN, request to mark target *)
            show*:     BOOLEAN;             (** IN, paired with `mark` *)
            type*:     Stores.TypeName;     (** IN *)
            isSingle*: BOOLEAN;             (** IN *)
            w*, h*:    INTEGER;             (** IN, view size; may be 0 *)
            rx*, ry*:  INTEGER;             (** IN, reference point *)
            dest*:     Views.Frame          (** OUT, preset NIL *)
        END;

        (** Actual drop — receiver should adopt the view. *)
        DropMsg* = RECORD (TransferMessage)
            view*:     Views.View;          (** IN *)
            isSingle*: BOOLEAN;             (** IN *)
            w*, h*:    INTEGER;             (** IN, proposed size *)
            rx*, ry*:  INTEGER              (** IN, reference point *)
        END;


        (** Abstract base for controllers.  Extends `Stores.Store`
            so a controller persists alongside its view (the wire
            format embeds the controller as an inline child store).
            Concrete controllers — `Containers.Controller` etc. —
            add the actual focus-routing and input handling. *)
        ControllerDesc* = ABSTRACT RECORD (Stores.StoreDesc) END;
        Controller*     = POINTER TO ControllerDesc;


    VAR
        (** Module-level focused view — set by HostWindows.FocusChild
            on EvFocus events, read by FocusView() below. *)
        focusedView: Views.View;


    (* -- ControllerDesc methods ------------------------------------------ *)

    (** Required by `Stores.Store` (ABSTRACT there).  Controllers
        without their own domain don't return one — concrete
        subclasses override. *)
    PROCEDURE (c: Controller) Domain* (): Stores.Domain;
    BEGIN
        RETURN NIL
    END Domain;

    (** BB-faithful focus query — returns the currently-focused
        view, or NIL when none. *)
    PROCEDURE FocusView* (): Views.View;
    BEGIN
        RETURN focusedView
    END FocusView;

    (** Set the currently focused view.  Called by the host
        windowing layer when an MDI child receives focus. *)
    PROCEDURE SetFocusView* (v: Views.View);
    BEGIN
        focusedView := v
    END SetFocusView;

    (** BB-faithful — current focus model.  Stub returns NIL until
        focus routing lands; framework callers tolerate NIL. *)
    PROCEDURE FocusModel* (): Models.Model;
    BEGIN
        RETURN NIL
    END FocusModel;


BEGIN
    focusedView := NIL
END Controllers.
