using System.Text;

static class OutputBuilder
{
    public static string GetOutputExtension(ExtractMode mode) => mode == ExtractMode.Markdown ? ".md" : ".txt";

    public static bool TryBuild(string filePath, byte[] data, ExtractMode mode, int minLength, int limit, MarkdownExportContext? markdownContext, out string output, out string? error)
    {
        if (mode == ExtractMode.Markdown)
        {
            return MarkdownDocumentExtractor.TryExtract(filePath, data, markdownContext, out output, out error);
        }

        output = Extractor.BuildOutput(filePath, data, mode, minLength, limit);
        error = string.Empty;
        return true;
    }
}

static class MarkdownDocumentExtractor
{
    public static bool TryExtract(string resolvedPath, byte[] data, MarkdownExportContext? context, out string markdown, out string? error)
    {
        try
        {
            var parser = new BlackBoxParser(data);
            var root = parser.ParseRoot();
            var model = FindFirstTextModel(root);
            if (model is null)
            {
                markdown = string.Empty;
                error = $"No standard text model found in {resolvedPath}.";
                return false;
            }

            markdown = MarkdownRenderer.Render(model, data, resolvedPath, context);
            error = string.Empty;
            return true;
        }
        catch (Exception ex)
        {
            markdown = string.Empty;
            error = $"Markdown extraction failed: {ex.Message}";
            return false;
        }
    }

    private static TextModelStore? FindFirstTextModel(StoreNode? root)
    {
        if (root is null)
        {
            return null;
        }

        var seen = new HashSet<StoreNode>(ReferenceEqualityComparer.Instance);
        var stack = new Stack<StoreNode>();
        stack.Push(root);

        while (stack.Count > 0)
        {
            var current = stack.Pop();
            if (!seen.Add(current))
            {
                continue;
            }

            if (current is TextModelStore textModel)
            {
                return textModel;
            }

            for (var index = current.Children.Count - 1; index >= 0; index--)
            {
                if (current.Children[index] is { } child)
                {
                    stack.Push(child);
                }
            }
        }

        return null;
    }
}

sealed class BlackBoxParser
{
    private const byte NilKind = 0x80;
    private const byte LinkKind = 0x81;
    private const byte StoreKind = 0x82;
    private const byte ElemKind = 0x83;
    private const byte NewLinkKind = 0x84;
    private const byte NewBaseKind = 0xF0;
    private const byte NewExtensionKind = 0xF1;
    private const byte OldTypeKind = 0xF2;

    private const string ContainersViewDesc = "Containers.ViewDesc";
    private const string ContainersModelDesc = "Containers.ModelDesc";
    private const string DocumentsModelDesc = "Documents.ModelDesc";
    private const string StdDocumentDesc = "Documents.StdDocumentDesc";
    private const string TextModelDesc = "TextModels.StdModelDesc";
    private const string TextAttrDesc = "TextModels.AttributesDesc";
    private const string LinkDesc = "StdLinks.LinkDesc";
    private const string TargetDesc = "StdLinks.TargetDesc";
    private const string RulerDesc = "TextRulers.RulerDesc";
    private const string StdRulerDesc = "TextRulers.StdRulerDesc";
    private const string StyleDesc = "TextRulers.StyleDesc";
    private const string StdStyleDesc = "TextRulers.StdStyleDesc";
    private const string RulerAttrDesc = "TextRulers.AttributesDesc";
    private const string ElementDesc = "Stores.ElemDesc";

    private readonly BufferReader _reader;
    private readonly Dictionary<int, string> _typeNames = [];
    private readonly Dictionary<int, int> _typeBaseIds = [];
    private readonly Dictionary<int, StoreNode> _elemStoreById = [];
    private readonly Dictionary<int, StoreNode> _storeById = [];

    private int _nextTypeId;
    private int _nextElemId;
    private int _nextStoreId;
    private int _lastNextPosition;
    private int _lastEndPosition;

    public BlackBoxParser(byte[] data)
    {
        _reader = new BufferReader(data);
    }

    public StoreNode ParseRoot()
    {
        if (_reader.Length < 8)
        {
            throw new InvalidOperationException("File is too short to be a BlackBox document.");
        }

        var magic = Encoding.ASCII.GetString(_reader.Data, 0, 4);
        if (magic is not "CDOo" and not "FCOo")
        {
            throw new InvalidOperationException($"Unsupported BlackBox file magic: {magic}");
        }

        _reader.Position = 8;
        return ReadStore() ?? throw new InvalidOperationException("Document root store is NIL.");
    }

    private StoreNode? ReadStore()
    {
        var kind = _reader.ReadByte();
        switch (kind)
        {
            case NilKind:
            {
                var comment = _reader.ReadInt32();
                var next = _reader.ReadInt32();
                _lastEndPosition = _reader.Position;
                _lastNextPosition = next > 0 || (next == 0 && (comment & 1) != 0)
                    ? _lastEndPosition + next
                    : 0;
                return null;
            }
            case LinkKind:
            {
                var id = _reader.ReadInt32();
                var comment = _reader.ReadInt32();
                var next = _reader.ReadInt32();
                _lastEndPosition = _reader.Position;
                _lastNextPosition = next > 0 || (next == 0 && (comment & 1) != 0)
                    ? _lastEndPosition + next
                    : 0;
                return _elemStoreById[id];
            }
            case NewLinkKind:
            {
                var id = _reader.ReadInt32();
                var comment = _reader.ReadInt32();
                var next = _reader.ReadInt32();
                _lastEndPosition = _reader.Position;
                _lastNextPosition = next > 0 || (next == 0 && (comment & 1) != 0)
                    ? _lastEndPosition + next
                    : 0;
                return _storeById[id];
            }
            case StoreKind:
            case ElemKind:
                return ReadObject(kind == ElemKind);
            default:
                throw new InvalidOperationException($"Unsupported store kind 0x{kind:X2} at offset {_reader.Position - 1}.");
        }
    }

    private StoreNode ReadObject(bool isElem)
    {
        var id = isElem ? _nextElemId++ : _nextStoreId++;
        var path = ReadPath();
        var primaryType = path.Count > 0 ? path[0] : "<unknown>";
        _reader.ReadInt32();
        var headerPos = _reader.Position;
        var next = _reader.ReadInt32();
        var down = _reader.ReadInt32();
        var payloadLength = _reader.ReadInt32();
        var payloadStart = _reader.Position;
        var payloadEnd = _reader.Position + payloadLength;
        var nextPosition = next > 0 ? headerPos + next + 4 : 0;
        var downPosition = down > 0 ? headerPos + down + 8 : 0;
        _lastNextPosition = nextPosition;
        _lastEndPosition = payloadEnd;

        var store = CreateStoreNode(primaryType, path, id);
        RegisterStore(store, isElem);

        ParseStorePayload(store, payloadStart, downPosition, payloadEnd);
        if (_reader.Position != payloadEnd)
        {
            if (_reader.Position > payloadEnd)
            {
                throw new InvalidOperationException($"Store parser overran payload for {primaryType} at offset {headerPos}.");
            }

            _reader.Position = payloadEnd;
        }

        return store;
    }

    private void RegisterStore(StoreNode store, bool isElem)
    {
        store.IsElement = isElem;
        if (isElem)
        {
            _elemStoreById[store.Id] = store;
        }
        else
        {
            _storeById[store.Id] = store;
        }
    }

    private List<string> ReadPath()
    {
        var path = new List<string>();
        var previousTypeId = -1;
        var kind = _reader.ReadByte();

        while (kind == NewExtensionKind)
        {
            var typeName = NormalizeTypeName(_reader.ReadXString());
            AddType(typeName, previousTypeId, _nextTypeId);
            previousTypeId = _nextTypeId;
            _nextTypeId++;
            if (!string.Equals(typeName, ElementDesc, StringComparison.Ordinal))
            {
                path.Add(typeName);
            }

            kind = _reader.ReadByte();
        }

        if (kind == NewBaseKind)
        {
            var typeName = NormalizeTypeName(_reader.ReadXString());
            AddType(typeName, previousTypeId, _nextTypeId);
            previousTypeId = _nextTypeId;
            _nextTypeId++;
            if (!string.Equals(typeName, ElementDesc, StringComparison.Ordinal))
            {
                path.Add(typeName);
            }
        }
        else if (kind == OldTypeKind)
        {
            var id = _reader.ReadInt32();
            if (previousTypeId >= 0)
            {
                _typeBaseIds[previousTypeId] = id;
            }

            while (id >= 0)
            {
                if (!_typeNames.TryGetValue(id, out var typeName))
                {
                    throw new InvalidOperationException($"Unknown type id {id} in type path.");
                }

                if (!string.Equals(typeName, ElementDesc, StringComparison.Ordinal))
                {
                    path.Add(typeName);
                }

                id = _typeBaseIds.TryGetValue(id, out var baseId) ? baseId : -1;
            }
        }
        else
        {
            throw new InvalidOperationException($"Unsupported type path marker 0x{kind:X2} at offset {_reader.Position - 1}.");
        }

        return path;
    }

    private void AddType(string typeName, int previousTypeId, int currentTypeId)
    {
        _typeNames[currentTypeId] = typeName;
        if (previousTypeId >= 0)
        {
            _typeBaseIds[previousTypeId] = currentTypeId;
        }
    }

    private static string NormalizeTypeName(string typeName)
    {
        if (typeName.EndsWith("^", StringComparison.Ordinal))
        {
            return typeName[..^1] + "Desc";
        }

        return typeName;
    }

    private StoreNode CreateStoreNode(string primaryType, IReadOnlyList<string> path, int id) => primaryType switch
    {
        TextModelDesc => new TextModelStore(id, path),
        TextAttrDesc => new TextAttributesStore(id, path),
        LinkDesc => new LinkStore(id, path),
        TargetDesc => new TargetStore(id, path),
        RulerDesc or StdRulerDesc => new RulerStore(id, path),
        StyleDesc or StdStyleDesc => new RulerStyleStore(id, path),
        RulerAttrDesc => new RulerAttributesStore(id, path),
        _ => new GenericStore(id, path)
    };

    private void ParseStorePayload(StoreNode store, int payloadStart, int downPosition, int payloadEnd)
    {
        switch (store)
        {
            case TextModelStore textModel:
                ParseTextModel(textModel, payloadEnd);
                return;
            case TextAttributesStore attributes:
                ParseTextAttributes(attributes);
                return;
            case LinkStore link:
                ParseLink(link);
                return;
            case TargetStore target:
                ParseTarget(target);
                return;
            case RulerStore ruler:
                ParseRuler(ruler, payloadEnd);
                return;
            case RulerStyleStore style:
                ParseRulerStyle(style, payloadEnd);
                return;
            case RulerAttributesStore rulerAttr:
                ParseRulerAttributes(rulerAttr);
                return;
            default:
                ParseGeneric(store, payloadStart, downPosition, payloadEnd);
                return;
        }
    }

    private void ParseGeneric(StoreNode store, int payloadStart, int downPosition, int payloadEnd)
    {
        ParseStoreBase(store);

        if (store.Path.Contains(ContainersViewDesc, StringComparer.Ordinal))
        {
            ParseViewsViewBase();
            ReadVersion();
            store.AddChild(ReadStore());
            store.AddChild(ReadStore());
            if (string.Equals(store.PrimaryType, StdDocumentDesc, StringComparison.Ordinal))
            {
                ReadVersion();
                _reader.ReadInt32(); _reader.ReadInt32(); _reader.ReadInt32(); _reader.ReadInt32(); _reader.ReadInt32(); _reader.ReadInt32();
                _reader.ReadBoolean();
            }
        }
        else if (store.Path.Contains(ContainersModelDesc, StringComparer.Ordinal))
        {
            ParseModelsModelBase();
            ReadVersion();
            if (string.Equals(store.PrimaryType, DocumentsModelDesc, StringComparison.Ordinal))
            {
                ReadVersion();
                store.AddChild(ReadStore());
                _reader.ReadInt32(); _reader.ReadInt32(); _reader.ReadInt32(); _reader.ReadInt32();
            }
        }

        if (store.Children.Count == 0 && downPosition > 0)
        {
            ReadEmbeddedChildren(store, payloadStart, downPosition, payloadEnd);
        }

        _reader.Position = payloadEnd;
    }

    private void ReadEmbeddedChildren(StoreNode store, int payloadStart, int downPosition, int payloadEnd)
    {
        var savedPosition = _reader.Position;
        var position = payloadStart;
        var nextPosition = downPosition;

        while (position < payloadEnd)
        {
            if (position < nextPosition)
            {
                position = nextPosition;
                continue;
            }

            if (position != nextPosition)
            {
                break;
            }

            _reader.Position = nextPosition;
            var child = ReadStore();
            store.AddChild(child);

            position = _lastEndPosition;
            nextPosition = _lastNextPosition > 0 ? _lastNextPosition : payloadEnd;
        }

        _reader.Position = savedPosition;
    }

    private void ParseViewsViewBase() => ReadVersion();

    private void ParseModelsModelBase() => ReadVersion();

    private void ParseStoreBase(StoreNode store)
    {
        ReadVersion();
        if (store.IsElement)
        {
            ReadVersion();
        }
    }

    private void ParseTextModel(TextModelStore store, int payloadEnd)
    {
        ParseStoreBase(store);
        ParseModelsModelBase();
        ReadVersion();
        ReadVersion();
        var stdModelVersion = ReadVersion();
        if (stdModelVersion is < 0 or > 1)
        {
            throw new InvalidOperationException($"Unsupported TextModels.StdModel version {stdModelVersion}.");
        }

        var descriptorLength = _reader.ReadInt32();
        var charOrigin = _reader.Position + descriptorLength;
        var attrDict = new List<TextAttributesStore?>();

        while (true)
        {
            var attrIndex = _reader.ReadByte();
            if (attrIndex == byte.MaxValue)
            {
                break;
            }

            TextAttributesStore? attr;
            if (attrIndex == attrDict.Count)
            {
                attr = ReadStore() as TextAttributesStore;
                attrDict.Add(attr);
                store.AddChild(attr);
            }
            else
            {
                attr = attrIndex < attrDict.Count ? attrDict[attrIndex] : null;
            }

            var runLength = _reader.ReadInt32();
            if (runLength > 0)
            {
                store.Runs.Add(new TextPieceRun(attr, charOrigin, runLength, false));
                charOrigin += runLength;
            }
            else if (runLength < 0)
            {
                var byteLength = -runLength;
                store.Runs.Add(new TextPieceRun(attr, charOrigin, byteLength, true));
                charOrigin += byteLength;
            }
            else
            {
                var width = _reader.ReadInt32();
                var height = _reader.ReadInt32();
                var view = ReadStore();
                store.AddChild(view);
                store.Runs.Add(new EmbeddedViewRun(attr, view, width, height));
                charOrigin += 1;
            }
        }

        _reader.Position = Math.Min(charOrigin, payloadEnd);
    }

    private void ParseTextAttributes(TextAttributesStore store)
    {
        ParseStoreBase(store);
        ReadVersion();
        store.Color = _reader.ReadInt32();
        _reader.ReadInt32();
        store.Typeface = _reader.ReadXString();
        store.Size = _reader.ReadInt32();
        store.StyleBits = _reader.ReadSet();
        store.Weight = _reader.ReadInt16();
        store.Offset = _reader.ReadInt32();
    }

    private void ParseLink(LinkStore store)
    {
        ParseStoreBase(store);
        ParseViewsViewBase();
        var version = ReadVersion();
        _reader.ReadBoolean();
        var len = _reader.ReadInt32();
        store.Command = len <= 0 ? null : _reader.ReadXString();
        store.LeftSide = store.Command is not null;
        if (store.LeftSide && version >= 1)
        {
            store.CloseMode = _reader.ReadInt32();
        }
    }

    private void ParseTarget(TargetStore store)
    {
        ParseStoreBase(store);
        ParseViewsViewBase();
        ReadVersion();
        _reader.ReadBoolean();
        var len = _reader.ReadInt32();
        store.Identifier = len <= 0 ? null : _reader.ReadXString();
        store.LeftSide = store.Identifier is not null;
    }

    private void ParseRuler(RulerStore store, int payloadEnd)
    {
        ParseStoreBase(store);
        ParseViewsViewBase();
        ReadVersion();
        store.Style = ReadStore() as RulerStyleStore;
        store.AddChild(store.Style);
        if (string.Equals(store.PrimaryType, StdRulerDesc, StringComparison.Ordinal))
        {
            ReadVersion();
        }

        _reader.Position = payloadEnd;
    }

    private void ParseRulerStyle(RulerStyleStore store, int payloadEnd)
    {
        ParseStoreBase(store);
        ParseModelsModelBase();
        ReadVersion();
        store.Attributes = ReadStore() as RulerAttributesStore;
        store.AddChild(store.Attributes);
        if (string.Equals(store.PrimaryType, StdStyleDesc, StringComparison.Ordinal))
        {
            ReadVersion();
        }

        _reader.Position = payloadEnd;
    }

    private void ParseRulerAttributes(RulerAttributesStore store)
    {
        ParseStoreBase(store);
        var version = ReadVersion();
        store.First = _reader.ReadInt32();
        store.Left = _reader.ReadInt32();
        store.Right = _reader.ReadInt32();
        store.Lead = _reader.ReadInt32();
        store.Asc = _reader.ReadInt32();
        store.Dsc = _reader.ReadInt32();
        store.Grid = _reader.ReadInt32();
        store.Options = _reader.ReadSet();
        var tabCount = _reader.ReadInt16();
        if (tabCount < 0)
        {
            tabCount = 0;
        }

        store.TabStops = new int[tabCount];
        for (var index = 0; index < tabCount; index++)
        {
            store.TabStops[index] = _reader.ReadInt32();
        }

        if (version >= 2)
        {
            store.TabTypes = new int[tabCount];
            for (var index = 0; index < tabCount; index++)
            {
                store.TabTypes[index] = _reader.ReadSet();
            }
        }
    }

    private int ReadVersion() => _reader.ReadByte();
}

static class MarkdownRenderer
{
    private const char Tab = (char)0x09;
    private const char Line = (char)0x0D;
    private const char Para = (char)0x0E;
    private const char ZwSpace = (char)0x8B;
    private const char NbSpace = (char)0x00A0;
    private const char DigitSpace = (char)0x008F;
    private const char Hyphen = (char)0x0090;
    private const char NbHyphen = (char)0x0091;
    private const char SoftHyphen = (char)0x00AD;
    private const int ItalicBit = 1 << 0;
    private const int UnderlineBit = 1 << 1;
    private const int BoldThreshold = 550;

    public static string Render(TextModelStore model, byte[] data, string sourcePath, MarkdownExportContext? context)
    {
        var sb = new StringBuilder();
        var style = default(StyleState);
        string? activeLink = null;
        var justBrokeParagraph = true;

        foreach (var run in model.Runs)
        {
            switch (run)
            {
                case TextPieceRun piece:
                    foreach (var ch in DecodePiece(piece, data))
                    {
                        if (ch is Para or Line or '\0')
                        {
                            CloseLink(sb, ref style, ref activeLink);

                            CloseStyles(sb, ref style);
                            TrimTrailingSpaces(sb);
                            if (!justBrokeParagraph)
                            {
                                sb.AppendLine().AppendLine();
                            }

                            justBrokeParagraph = true;
                            continue;
                        }

                        ApplyStyle(sb, piece.Attributes, ref style);
                        justBrokeParagraph = false;

                        switch (ch)
                        {
                            case Tab:
                                sb.Append("    ");
                                break;
                            case NbSpace:
                            case DigitSpace:
                                sb.Append(' ');
                                break;
                            case ZwSpace:
                                break;
                            case Hyphen:
                            case NbHyphen:
                            case SoftHyphen:
                                sb.Append('-');
                                break;
                            default:
                                sb.Append(ch);
                                break;
                        }
                    }
                    break;
                case EmbeddedViewRun embedded:
                    switch (embedded.View)
                    {
                        case LinkStore { LeftSide: true, Command: { } command }:
                            ApplyStyle(sb, embedded.Attributes, ref style);
                            sb.Append('[');
                            activeLink = ConvertLink(command, sourcePath, context);
                            justBrokeParagraph = false;
                            break;
                        case LinkStore { LeftSide: false }:
                            CloseLink(sb, ref style, ref activeLink);
                            break;
                        case TargetStore { LeftSide: true, Identifier: { } ident }:
                            CloseStyles(sb, ref style);
                            sb.Append("<a id=\"").Append(EscapeHtml(ident)).Append("\"></a>");
                            justBrokeParagraph = false;
                            break;
                        default:
                            break;
                    }
                    break;
            }
        }

        CloseLink(sb, ref style, ref activeLink);

        CloseStyles(sb, ref style);
        TrimTrailingSpaces(sb);
        if (sb.Length == 0 || sb[^1] != '\n')
        {
            sb.AppendLine();
        }

        return sb.ToString();
    }

    private static IEnumerable<char> DecodePiece(TextPieceRun piece, byte[] data)
    {
        if (!piece.IsWide)
        {
            for (var index = 0; index < piece.ByteLength; index++)
            {
                yield return (char)data[piece.Offset + index];
            }

            yield break;
        }

        for (var index = 0; index < piece.ByteLength; index += 2)
        {
            yield return (char)(data[piece.Offset + index] | (data[piece.Offset + index + 1] << 8));
        }
    }

    private static string ConvertLink(string command, string currentSourcePath, MarkdownExportContext? context)
    {
        const string targetPrefix = "StdLinks.ShowTarget('";
        if (command.StartsWith(targetPrefix, StringComparison.Ordinal) && command.EndsWith("')", StringComparison.Ordinal))
        {
            return "#" + command[targetPrefix.Length..^2];
        }

        const string browserPrefix = "StdCmds.OpenBrowser('";
        const string docPrefix = "StdCmds.OpenDoc('";
        string? commandPrefix = null;
        if (command.StartsWith(browserPrefix, StringComparison.Ordinal))
        {
            commandPrefix = browserPrefix;
        }
        else if (command.StartsWith(docPrefix, StringComparison.Ordinal))
        {
            commandPrefix = docPrefix;
        }

        if (commandPrefix is not null)
        {
            var start = commandPrefix.Length;
            var end = command.IndexOf('\'', start);
            if (end > start)
            {
                var target = command[start..end];
                var targetRelativePath = target.Replace('/', Path.DirectorySeparatorChar).Replace('\\', Path.DirectorySeparatorChar);
                if (!targetRelativePath.EndsWith(".odc", StringComparison.OrdinalIgnoreCase))
                {
                    targetRelativePath += ".odc";
                }

                if (!string.IsNullOrWhiteSpace(context?.ReviewRootPath) && !string.IsNullOrWhiteSpace(context?.OutputPath))
                {
                    var targetOutputPath = Path.Combine(context.ReviewRootPath!, targetRelativePath + ".md");
                    var currentOutputDirectory = Path.GetDirectoryName(context.OutputPath!);
                    if (!string.IsNullOrWhiteSpace(currentOutputDirectory))
                    {
                        return Path.GetRelativePath(currentOutputDirectory, targetOutputPath).Replace('\\', '/');
                    }
                }

                return target.Replace(" ", "%20", StringComparison.Ordinal);
            }
        }

        return command;
    }

    private static void CloseLink(StringBuilder sb, ref StyleState style, ref string? activeLink)
    {
        if (activeLink is null)
        {
            return;
        }

        CloseStyles(sb, ref style);
        sb.Append("](").Append(activeLink).Append(')');
        activeLink = null;
    }

    private static string EscapeHtml(string value) => value.Replace("&", "&amp;", StringComparison.Ordinal).Replace("\"", "&quot;", StringComparison.Ordinal).Replace("<", "&lt;", StringComparison.Ordinal).Replace(">", "&gt;", StringComparison.Ordinal);

    private static void ApplyStyle(StringBuilder sb, TextAttributesStore? attributes, ref StyleState current)
    {
        var next = StyleState.From(attributes);
        if (current.Equals(next))
        {
            return;
        }

        CloseStyles(sb, ref current);
        if (next.Bold)
        {
            sb.Append("**");
        }

        if (next.Italic)
        {
            sb.Append('*');
        }

        if (next.Underline)
        {
            sb.Append("<u>");
        }

        current = next;
    }

    private static void CloseStyles(StringBuilder sb, ref StyleState current)
    {
        if (current.Underline)
        {
            sb.Append("</u>");
        }

        if (current.Italic)
        {
            sb.Append('*');
        }

        if (current.Bold)
        {
            sb.Append("**");
        }

        current = default;
    }

    private static void TrimTrailingSpaces(StringBuilder sb)
    {
        while (sb.Length > 0 && (sb[^1] == ' ' || sb[^1] == '\t'))
        {
            sb.Length--;
        }
    }

    private readonly record struct StyleState(bool Bold, bool Italic, bool Underline)
    {
        public static StyleState From(TextAttributesStore? attributes)
        {
            if (attributes is null)
            {
                return default;
            }

            return new StyleState(
                Bold: attributes.Weight >= BoldThreshold,
                Italic: (attributes.StyleBits & ItalicBit) != 0,
                Underline: (attributes.StyleBits & UnderlineBit) != 0);
        }
    }
}

internal sealed record MarkdownExportContext(string? SourceRootPath, string? ReviewRootPath, string? OutputPath);
sealed class BufferReader(byte[] data)
{
    public byte[] Data { get; } = data;
    public int Position { get; set; }
    public int Length => Data.Length;

    public byte ReadByte()
    {
        if (Position >= Data.Length)
        {
            throw new InvalidOperationException("Unexpected end of file.");
        }

        return Data[Position++];
    }

    public bool ReadBoolean() => ReadByte() != 0;

    public short ReadInt16()
    {
        EnsureAvailable(2);
        var value = (short)(Data[Position] | (Data[Position + 1] << 8));
        Position += 2;
        return value;
    }

    public int ReadInt32()
    {
        EnsureAvailable(4);
        var value = Data[Position]
            | (Data[Position + 1] << 8)
            | (Data[Position + 2] << 16)
            | (Data[Position + 3] << 24);
        Position += 4;
        return value;
    }

    public int ReadSet() => ReadInt32();

    public string ReadXString()
    {
        var start = Position;
        while (Position < Data.Length && Data[Position] != 0)
        {
            Position++;
        }

        if (Position >= Data.Length)
        {
            throw new InvalidOperationException("Unterminated XString.");
        }

        var value = Encoding.Latin1.GetString(Data, start, Position - start);
        Position++;
        return value;
    }

    private void EnsureAvailable(int count)
    {
        if (Position + count > Data.Length)
        {
            throw new InvalidOperationException("Unexpected end of file.");
        }
    }
}

abstract class StoreNode(int id, IReadOnlyList<string> path)
{
    public int Id { get; } = id;
    public IReadOnlyList<string> Path { get; } = path;
    public string PrimaryType => Path.Count > 0 ? Path[0] : "<unknown>";
    public List<StoreNode> Children { get; } = [];
    public bool IsElement { get; set; }

    public void AddChild(StoreNode? child)
    {
        if (child is not null)
        {
            Children.Add(child);
        }
    }
}

sealed class GenericStore(int id, IReadOnlyList<string> path) : StoreNode(id, path);

sealed class TextModelStore(int id, IReadOnlyList<string> path) : StoreNode(id, path)
{
    public List<TextRun> Runs { get; } = [];
}

sealed class TextAttributesStore(int id, IReadOnlyList<string> path) : StoreNode(id, path)
{
    public int Color { get; set; }
    public string Typeface { get; set; } = string.Empty;
    public int Size { get; set; }
    public int StyleBits { get; set; }
    public int Weight { get; set; }
    public int Offset { get; set; }
}

sealed class LinkStore(int id, IReadOnlyList<string> path) : StoreNode(id, path)
{
    public bool LeftSide { get; set; }
    public string? Command { get; set; }
    public int CloseMode { get; set; }
}

sealed class TargetStore(int id, IReadOnlyList<string> path) : StoreNode(id, path)
{
    public bool LeftSide { get; set; }
    public string? Identifier { get; set; }
}

sealed class RulerStore(int id, IReadOnlyList<string> path) : StoreNode(id, path)
{
    public RulerStyleStore? Style { get; set; }
}

sealed class RulerStyleStore(int id, IReadOnlyList<string> path) : StoreNode(id, path)
{
    public RulerAttributesStore? Attributes { get; set; }
}

sealed class RulerAttributesStore(int id, IReadOnlyList<string> path) : StoreNode(id, path)
{
    public int First { get; set; }
    public int Left { get; set; }
    public int Right { get; set; }
    public int Lead { get; set; }
    public int Asc { get; set; }
    public int Dsc { get; set; }
    public int Grid { get; set; }
    public int Options { get; set; }
    public int[] TabStops { get; set; } = [];
    public int[] TabTypes { get; set; } = [];
}

abstract record TextRun(TextAttributesStore? Attributes);

sealed record TextPieceRun(TextAttributesStore? Attributes, int Offset, int ByteLength, bool IsWide) : TextRun(Attributes);

sealed record EmbeddedViewRun(TextAttributesStore? Attributes, StoreNode? View, int Width, int Height) : TextRun(Attributes);