MODULE NewModels;

*(*  *)*

    IMPORT Stores, Models;

    CONST minVersion = 0; maxVersion = 0;

    TYPE

        **Model*** = POINTER TO ABSTRACT RECORD (Models.Model) END;

        **Directory*** = POINTER TO ABSTRACT RECORD END;

        **UpdateMsg*** = RECORD (Models.UpdateMsg)

            **(* message fields *)**

        END;

        StdModel = POINTER TO RECORD (Model)

            **(* model fields *)**

        END;

        StdDirectory = POINTER TO RECORD (Directory) END;

        Op = POINTER TO RECORD (Stores.Operation)

            model: StdModel;

            **(* model-operation fields *)**

        END;

    VAR **dir**-, **stdDir**-: Directory;

    *(** Model **)*

    *(** Directory **)*

    PROCEDURE (d: Directory) **New*** (): Model, NEW, ABSTRACT;

    *(* Op *)*

    PROCEDURE (op: Op) Do;

        VAR msg: UpdateMsg;

    BEGIN

        **(* perform model operation and set up the fields of the update message accordingly *)**

        Models.Broadcast(op.model, msg)    *(* update all views on this model *)*

    END Do;

    PROCEDURE NewOp (model: StdModel **(* additional parameters *)** ): Op;

        VAR op: Op;

    BEGIN

        ASSERT(model # NIL, 100);

        NEW(op); op.model := model;

        **(* set up operation parameters *)**

        **RETURN** op

    END NewOp;

    *(* StdModel *)*

    PROCEDURE (m: StdModel) Internalize (VAR rd: Stores.Reader);

        VAR version: INTEGER;

    BEGIN

        *(* m is not initalized *)*

*        (* m.Domain() = NIL *)*

        m.Internalize^(rd);

        IF ~rd.cancelled THEN

            rd.ReadVersion(minVersion, maxVersion, version);

            IF ~rd.cancelled THEN

                **(* read model fields *)**

            END

        END

    END Internalize;

    PROCEDURE (m: StdModel) Externalize (VAR wr: Stores.Writer);

    BEGIN

        *(* m is initalized *)*

        m.Externalize^(wr);

        wr.WriteVersion(maxVersion);

        **(* write model fields *)**

    END Externalize;

    PROCEDURE (m: StdModel) CopyFrom (source: Stores.Store);

    BEGIN

        *(* m is not yet initialized *)*

*        (* m.Domain() = NIL *)*

        *(* source # NIL *)*

        *(* TYP(source) = TYP(m) *)*

        WITH source: StdModel DO

            **(* perform deep copy of source *)**

        END

    END CopyFrom;

    *(* StdDirectory *)*

    PROCEDURE (d: StdDirectory) New (): Model;

        VAR m: StdModel;

    BEGIN

        NEW(m);

**        (* initialize m *)**;

        **RETURN** m

    END New;

    *(** miscellaneous **)*

    PROCEDURE **SetDir*** (d: Directory);

    BEGIN

        ASSERT(d # NIL, 20);

        dir := d

    END SetDir;

    PROCEDURE Init;

        VAR d: StdDirectory;

    BEGIN

        NEW(d); stdDir := d; dir := d

    END Init;

BEGIN

    Init

END NewModels.

