MODULE M_Module_BEGIN_Block_Runs;
    VAR seed: INTEGER;

    PROCEDURE Run* (): INTEGER;
    BEGIN RETURN seed END Run;

BEGIN
    seed := 99
END M_Module_BEGIN_Block_Runs.
