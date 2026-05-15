MODULE ExtendModelsUpdateMsgProbe;
IMPORT Stores, Models, Containers, Properties, Ports, Fonts, Views;

TYPE
    OtherDesc* = RECORD (Stores.StoreDesc) x*: INTEGER END;
    Bar* = RECORD (Models.UpdateMsg) op*: INTEGER END;

END ExtendModelsUpdateMsgProbe.
