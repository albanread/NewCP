MODULE WinSpec;
(**
   WinSpec — declarative window layout builder for wingui spec_bind.

   The builder is backed by a Rust thread-local JSON string builder.
   Typical usage:

     WinSpec.Begin("My App");
     WinSpec.AddTextarea("output", "Output", "", TRUE);
     WinSpec.GetSpec(spec, LEN(spec));
     HostWindows.PublishUi(spec);

   Nesting containers:

     WinSpec.Begin("My App");
     WinSpec.OpenRow(-1);
       WinSpec.AddButton("run", "Run", "run");
       WinSpec.AddButton("clear", "Clear", "clear");
     WinSpec.CloseContainer;
     WinSpec.AddTextarea("output", "Output", "", TRUE);
     WinSpec.GetSpec(spec, LEN(spec));

   UpdateTextarea patches a single textarea's value without rebuilding the
   whole spec.  Pass the spec buffer (same ARRAY OF SHORTCHAR), the widget id,
   and the new text.  Returns 1 on success.
*)

(**
   Begin — reset the builder with a new window title.
*)
PROCEDURE Begin*(title: ARRAY OF SHORTCHAR);
BEGIN
END Begin;

(**
   OpenStack — open a vertical stack container.
   gap: spacing between children in pixels, or -1 for the default.
*)
PROCEDURE OpenStack*(gap: INTEGER);
BEGIN
END OpenStack;

(**
   OpenRow — open a horizontal row container.
   gap: spacing between children in pixels, or -1 for the default.
*)
PROCEDURE OpenRow*(gap: INTEGER);
BEGIN
END OpenRow;

(**
   CloseContainer — close the most recently opened container.
*)
PROCEDURE CloseContainer*;
BEGIN
END CloseContainer;

(**
   AddTextarea — add a multi-line text area.
   id:       widget identifier (used in events and UpdateTextarea)
   label:    visible label above the widget
   value:    initial text content
   readonly: non-zero to prevent user editing
*)
PROCEDURE AddTextarea*(id, label, value: ARRAY OF SHORTCHAR; readonly: INTEGER);
BEGIN
END AddTextarea;

(**
   AddButton — add a clickable button.
   id:    widget identifier
   label: button text
   event: event name fired when the button is clicked
*)
PROCEDURE AddButton*(id, label, event: ARRAY OF SHORTCHAR);
BEGIN
END AddButton;

(**
   AddText — add a static text label.
*)
PROCEDURE AddText*(text: ARRAY OF SHORTCHAR);
BEGIN
END AddText;

(**
   GetSpec — copy the built JSON spec into buf.
   Returns 1 on success, 0 if buf is too small.
*)
PROCEDURE GetSpec*(VAR buf: ARRAY OF SHORTCHAR): INTEGER;
BEGIN
  RETURN 0
END GetSpec;

(**
   UpdateTextarea — patch one textarea's value inside an existing spec buffer.
   spec:  the spec buffer (modified in-place)
   id:    the textarea widget id
   value: the new text value
   Returns 1 on success, 0 on failure.
*)
PROCEDURE UpdateTextarea*(VAR spec: ARRAY OF SHORTCHAR;
                          id, value: ARRAY OF SHORTCHAR): INTEGER;
BEGIN
  RETURN 0
END UpdateTextarea;

END WinSpec.
