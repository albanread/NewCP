MODULE ImportedConstProbe;
IMPORT ImportedConstProbeBase;
CONST
    derived = ImportedConstProbeBase.base DIV 2;

PROCEDURE Run* (): INTEGER;
BEGIN
    RETURN derived
END Run;
END ImportedConstProbe.
