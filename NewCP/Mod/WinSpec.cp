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

PROCEDURE GetSpec*(VAR buf: ARRAY OF SHORTCHAR): INTSHORT;
BEGIN
  RETURN 0
END GetSpec;

END WinSpec.
