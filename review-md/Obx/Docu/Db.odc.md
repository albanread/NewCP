**Overview by Example: ObxDb**

This example provides two commands to the user. The first, *EnterData*, takes a text selection as input, and reads it line by line. Each line should consist of an integer number, followed by a string, trailed by a real number. Each such tuple is entered into a globally anchored linear list, sorted by the integer value. The second command, *ListData*, generates a text which displays the data currently in the list.

[<u>ObxDb  sources</u>](../Mod/Db.odc.md)

 ObxDb.EnterData         ObxDb.ListData         ObxDb.Reset

To try out the example, select the following lines, and then click the left commander above:

1    Cray    14.8

3    NEC    16.6

2    IBM    8.3

Now click the middle commander, as a result a window opens with the sorted input. If you repeat both steps, you'll note that the input has been added to the list a second time, and that consequently every item appears twice in the output.

This example has shown how a text can be scanned symbol by symbol, instead of character by character.

