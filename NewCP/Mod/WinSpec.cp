MODULE WinSpec;

PROCEDURE Begin*(title: ARRAY OF SHORTCHAR);
BEGIN
END Begin;

PROCEDURE OpenStack*(gap: INTSHORT);
BEGIN
END OpenStack;

PROCEDURE OpenRow*(gap: INTSHORT);
BEGIN
END OpenRow;

PROCEDURE CloseContainer*;
BEGIN
END CloseContainer;

PROCEDURE AddButton*(id, label, event: ARRAY OF SHORTCHAR);
BEGIN
END AddButton;

PROCEDURE AddText*(text: ARRAY OF SHORTCHAR);
BEGIN
END AddText;

PROCEDURE AddTextarea*(id, label, value: ARRAY OF SHORTCHAR; readonly: INTSHORT);
BEGIN
END AddTextarea;

PROCEDURE AddTextGrid*(id, event: ARRAY OF SHORTCHAR; cols, rows: INTSHORT);
BEGIN
END AddTextGrid;

PROCEDURE AddSurface*(id, event: ARRAY OF SHORTCHAR; width, height: INTSHORT);
BEGIN
END AddSurface;

PROCEDURE AddRgbaPane*(id, event: ARRAY OF SHORTCHAR; width, height: INTSHORT);
BEGIN
END AddRgbaPane;

PROCEDURE GetSpec*(VAR buf: ARRAY OF SHORTCHAR): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END GetSpec;

END WinSpec.
