MODULE ObxAddress0;

*(***

*    project    = "BlackBox"*

*    organization    = "www.oberon.ch"*

*    contributors    = "Oberon microsystems"*

*    version    = "[**<u>System/Rsrc/About</u>*](StdCmds.OpenToolDialog('System/Rsrc/About', 'About BlackBox'))*"*

*    copyright    = "[**<u>System/Rsrc/About</u>*](StdCmds.OpenToolDialog('System/Rsrc/About', 'About BlackBox'))*"*

*    license    = "[**<u>Docu/BB-License</u>*](../../Docu/BB-License.odc.md)*"*

*    changes    = ""*

*    issues    = ""*

***)*

    VAR

        **adr***: RECORD

            name*:    ARRAY 64 OF CHAR;

            city*:    ARRAY 24 OF CHAR;

            country*:    ARRAY 16 OF CHAR;

            customer*:    INTEGER;

            update*:    BOOLEAN

        END;

    PROCEDURE **OpenText***;

    BEGIN



    END OpenText;



END ObxAddress0.

