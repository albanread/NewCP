MODULE Properties;
(*
   First slice of the BlackBox `Properties` port.

   `Properties` is the property-bag / preference-bag layer that
   sits between views and the dialog/UI runtime — it carries the
   "every view publishes a sorted list of properties; preferences
   round-trip through the same channel" shape.  The full module
   (~720 lines) is two layered concerns:

   1. The property / preference TYPE TREE — pure data records
      that subclassing modules (TextViews / FormViews / Containers /
      …) extend to expose their typed-state.  Extending the
      `Preference` chain is how a parent view asks a child "what
      bounds / what size / what focus state do you want?".

   2. A bundle of module-level procedures (`Insert`, `CopyOfList`,
      `Merge`, `Intersect`, `PreferredSize`, `ThisType`, …) that
      operate on `Property` lists and `Preference` message
      round-trips.  Those rely on `Kernel.Type` (RTTI) for
      type-tagged sorting and on `Math` / `Services` / `Dialog`
      for the geometry helpers and notification hooks.

   This slice ships (1) plus a `Property.IntersectWith` ABSTRACT
   stub so concrete property classes can override.  The module
   procedures are deferred (they need `Kernel.Type` / `Services` /
   `Math` / `Dialog` proper, none of which are fully ported).
*)

    IMPORT Stores, Fonts, Views, Controllers, Dialog;

    CONST
        (** StdProp.known / valid bitmask positions. *)
        color*    = 0;
        typeface* = 1;
        size*     = 2;
        style*    = 3;
        weight*   = 4;

        (** SizeProp.known / valid bitmask positions. *)
        width*  = 0;
        height* = 1;

        (** PollVerbsMsg limit on verbs per poll. *)
        maxVerbs* = 16;

        (** PollPickMsg.mark / PollPick mark flag. *)
        noMark* = FALSE;
        mark*   = TRUE;

        (** PollPickMsg.show / PollPick show flag. *)
        hide* = FALSE;
        show* = TRUE;


    TYPE
        (** Sorted property-list head.  Concrete property classes
            (`StdProp`, `SizeProp`, framework-specific extensions)
            extend this and override `IntersectWith` so a
            properties-merge can short-circuit on known/valid
            bitmasks. *)
        PropertyDesc* = ABSTRACT RECORD
            next-:              Property;  (** sorted by TypeDesc address *)
            known*, readOnly*:  SET;       (** polling masks *)
            valid*:             SET
        END;
        Property* = POINTER TO PropertyDesc;

        (** Concrete "standard" property — the typed bag every
            text-style-aware view publishes.  Five settable axes:
            color, typeface, size, style (val + mask SET pair),
            weight. *)
        StdPropDesc* = RECORD (PropertyDesc)
            color*:    Dialog.Color;
            typeface*: Fonts.Typeface;
            size*:     INTEGER;
            style*:    RECORD val*, mask*: SET END;
            weight*:   INTEGER
        END;
        StdProp* = POINTER TO StdPropDesc;

        (** Geometric size property — preset/queried (width,
            height) in user units. *)
        SizePropDesc* = RECORD (PropertyDesc)
            width*, height*: INTEGER
        END;
        SizeProp* = POINTER TO SizePropDesc;


        (** Re-export the Views property-message base.  All
            property-flavoured messages below extend this so the
            handler chain can pattern-match on
            `Properties.Message`. *)
        Message* = Views.PropMessage;

        (** Read the current property bag from a focused View. *)
        PollMsg* = RECORD (Message)
            prop*: Property              (** preset NIL *)
        END;

        (** Apply a new property bag, optionally with an old
            value for undo. *)
        SetMsg* = RECORD (Message)
            old*, prop*: Property
        END;


        (** Abstract Preference base — the round-trip shape that
            asks a parent view for its preferred geometry / focus /
            type state for the child being initialised. *)
        Preference* = ABSTRACT RECORD (Message) END;

        (** "How should you be resized?"  Both axes plus
            fit-to-page / fit-to-window flags. *)
        ResizePref* = RECORD (Preference)
            fixed*:        BOOLEAN;
            horFitToPage*: BOOLEAN;
            verFitToPage*: BOOLEAN;
            horFitToWin*:  BOOLEAN;
            verFitToWin*:  BOOLEAN
        END;

        (** "What size do you want me at?"  Caller presets
            `(w, h)` to its preference; receiver may shrink or
            grow.  `fixedW` / `fixedH` constrain the receiver. *)
        SizePref* = RECORD (Preference)
            w*, h*:           INTEGER;
            fixedW*, fixedH*: BOOLEAN
        END;

        (** "How big can you grow?" — sets `(w, h)` to maxima.
            Each defaults to Views.undefined. *)
        BoundsPref* = RECORD (Preference)
            w*, h*: INTEGER
        END;

        (** "Should you accept focus on this click?"  At-location
            variant carries cursor coords; hot/set focus are OUT. *)
        FocusPref* = RECORD (Preference)
            atLocation*:        BOOLEAN;
            x*, y*:             INTEGER;
            hotFocus*, setFocus*: BOOLEAN
        END;

        (** Keystroke routing.  `char` + `focus` are IN;
            getFocus / accepts say whether the receiver wants
            focus and whether it consumes the key. *)
        ControlPref* = RECORD (Preference)
            char*:      CHAR;
            focus*:     Views.View;
            getFocus*:  BOOLEAN;
            accepts*:   BOOLEAN
        END;

        (** "Have you seen a view of this type?"  Used by the
            inverse type-registry walk; returns first match
            in `view`. *)
        TypePref* = RECORD (Preference)
            type*: Stores.TypeName;
            view*: Views.View
        END;


        (** Verb-polling message.  Receiver fills in the label
            and disabled/checked state for a UI button bound to
            `verb`. *)
        PollVerbMsg* = RECORD (Message)
            verb*:                INTEGER;
            label*:               ARRAY 64 OF CHAR;
            disabled*, checked*:  BOOLEAN
        END;

        (** Verb-action message — sent when the user invokes the
            verb (clicks the button). *)
        DoVerbMsg* = RECORD (Message)
            verb*:  INTEGER;
            frame*: Views.Frame
        END;


        (** Controller-side wrappers.  These ride the Controllers
            message channel (which is `Views.CtrlMessage`-typed)
            and carry the properties round-trip payload. *)
        CollectMsg* = RECORD (Controllers.Message)
            poll*: PollMsg
        END;

        EmitMsg* = RECORD (Controllers.RequestMessage)
            set*: SetMsg
        END;

        PollPickMsg* = RECORD (Controllers.TransferMessage)
            mark*: BOOLEAN;
            show*: BOOLEAN;
            dest*: Views.Frame
        END;

        PickMsg* = RECORD (Controllers.TransferMessage)
            prop*: Property
        END;


    VAR
        (** Bumped on every Property-list change — view focus
            caches refresh when their snapshot's `era` lags. *)
        era-: INTEGER;


    (* -- PropertyDesc methods -------------------------------------------- *)

    (** Intersect this property bag with `q`, narrowing the valid
        SET to bits both agree on.  ABSTRACT — `StdProp` and
        `SizeProp` override.  `equal` is set to FALSE if any axis
        differed, so the caller knows to mark its receiver dirty. *)
    PROCEDURE (p: Property) IntersectWith* (q: Property; OUT equal: BOOLEAN), NEW, ABSTRACT;

END Properties.
