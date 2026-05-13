MODULE ExtendModelsUpdateMsgProbe;
IMPORT HostStores, Stores, Models, Containers, Properties, Ports, Fonts, Views;

TYPE
    OtherDesc* = RECORD (HostStores.StoreDesc) x*: INTEGER END;
    Bar* = RECORD (Models.UpdateMsg) op*: INTEGER END;

END ExtendModelsUpdateMsgProbe.
