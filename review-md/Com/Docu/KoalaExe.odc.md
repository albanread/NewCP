<a id="7.3"></a>**ComKoalaExe Example**

This version of the Koala example can be linked as an independent exe. It can be tested with the ObjUser.exe program which is provided with the book "Inside OLE". Note that you have to set the ObjUser.exe program into EXE mode.

The EXE file opens its application window which is closed as soon as the last interface pointer is released. It also must implement a main loop which polls and dispatches all Windows events. Additionally, the body of the module must register the factory upon start up and remove the factory from the registry as soon as the EXE is unloaded.

The ComKoalaExe example can be linked to EKoala1.EXE with the following command:

    DevLinker.LinkExe "Com/EKoala1.exe" := Kernel+ ComKoalaExe ~

The registry must be informed about which program to start if a local server for our IKoala factory ({00021146-0000-0000-C000-000000000046}) is requested. The necessary registry file[<u>Com/Reg/EKoala1.reg</u>](../Reg/EKoala1.reg.odc.md) is given below. Use the REGEDIT tool to update the registry. Adjust path names if necessary!

REGEDIT

HKEY_CLASSES_ROOT\Koala1.0 = Koala Object Chapter 5

HKEY_CLASSES_ROOT\Koala1.0\CLSID = {00021146-0000-0000-C000-000000000046}

HKEY_CLASSES_ROOT\Koala = Koala Object Chapter 5

HKEY_CLASSES_ROOT\Koala\CurVer = Koala1.0

HKEY_CLASSES_ROOT\Koala\CLSID = {00021146-0000-0000-C000-000000000046}

HKEY_CLASSES_ROOT\CLSID\{00021146-0000-0000-C000-000000000046} = Koala Object Chapter 5

HKEY_CLASSES_ROOT\CLSID\{00021146-0000-0000-C000-000000000046}\ProgID = Koala1.0

HKEY_CLASSES_ROOT\CLSID\{00021146-0000-0000-C000-000000000046}\VersionIndependentProgID = Koala

HKEY_CLASSES_ROOT\CLSID\{00021146-0000-0000-C000-000000000046}\LocalServer32 = C:\BlackBox\Com\Ekoala1.exe

HKEY_CLASSES_ROOT\CLSID\{00021146-0000-0000-C000-000000000046}\NotInsertable

[<u>ComKoalaExe  sources</u>](../Mod/KoalaExe.odc.md)

