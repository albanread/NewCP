MODULE StdCFrames;
(*
   First slice of the BlackBox `StdCFrames` port.

   BB's StdCFrames is the abstract base class for control
   frames — the per-control rendering helpers that StdControls
   and StdScrollers extend.  ~430 lines.

   This slice ships only the type surface — concrete frames
   live in their owning Std/* modules.  Welcome-page open
   doesn't reach controls, so no body needed.
*)

    IMPORT Fonts, Ports, Views;


    TYPE
        (** Abstract control frame — Std/Controls and
            Std/Scrollers extend this.  Carries a font / port
            pair for its drawing helpers. *)
        FrameDesc* = ABSTRACT RECORD (Views.FrameDesc)
            font*: Fonts.Font;
            color*: Ports.Color
        END;
        Frame* = POINTER TO FrameDesc;

END StdCFrames.
