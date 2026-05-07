MODULE HostWindows;

PROCEDURE PublishUi*(json: ARRAY OF SHORTCHAR);
BEGIN
END PublishUi;

PROCEDURE RequestClose*;
BEGIN
END RequestClose;

PROCEDURE RequestPresent*;
BEGIN
END RequestPresent;

PROCEDURE WaitNamedEvent*(VAR name: ARRAY OF SHORTCHAR;
                          VAR payload: ARRAY OF SHORTCHAR;
                          timeoutMs: INTEGER): INTSHORT;
  VAR ok: INTSHORT;
BEGIN
  ok := SHORT(0);
  RETURN ok
END WaitNamedEvent;

END HostWindows.
