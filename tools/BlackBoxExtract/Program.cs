using System.Text;
using System.Text.RegularExpressions;

Encoding.RegisterProvider(CodePagesEncodingProvider.Instance);

var options = Options.Parse(args);
return options.Execute();

enum ExtractMode
{
    Summary,
    Strings,
    Source,
    Review,
    Markdown
}

sealed class Options
{
    private static readonly string[] DefaultExtensions = [".odc", ".ocf", ".osf"];

    public required string TargetPath { get; init; }
    public required bool BatchMode { get; init; }
    public string ReviewRoot { get; init; } = "review";
    public ExtractMode Mode { get; init; } = ExtractMode.Review;
    public int MinLength { get; init; } = 4;
    public int Limit { get; init; }
    public IReadOnlyList<string> Extensions { get; init; } = DefaultExtensions;
    public string? OutputPath { get; init; }

    public static Options Parse(string[] args)
    {
        if (args.Length == 0)
        {
            throw new ArgumentException(UsageText);
        }

        string? targetPath = null;
        string reviewRoot = "review";
        ExtractMode mode = ExtractMode.Review;
        int minLength = 4;
        int limit = 0;
        string? outputPath = null;
        bool batchMode = false;
        var extensions = new List<string>(DefaultExtensions);

        for (var index = 0; index < args.Length; index++)
        {
            var arg = args[index];
            if (!arg.StartsWith("-", StringComparison.Ordinal))
            {
                targetPath ??= arg;
                continue;
            }

            string NextValue()
            {
                if (index + 1 >= args.Length)
                {
                    throw new ArgumentException($"Missing value for {arg}");
                }

                index++;
                return args[index];
            }

            switch (arg)
            {
                case "--batch":
                    batchMode = true;
                    break;
                case "--review-root":
                    reviewRoot = NextValue();
                    break;
                case "--mode":
                    mode = NextValue().ToLowerInvariant() switch
                    {
                        "summary" => ExtractMode.Summary,
                        "strings" => ExtractMode.Strings,
                        "source" => ExtractMode.Source,
                        "review" => ExtractMode.Review,
                        "markdown" => ExtractMode.Markdown,
                        var value => throw new ArgumentException($"Unknown mode: {value}")
                    };
                    break;
                case "--min-length":
                    minLength = int.Parse(NextValue());
                    break;
                case "--limit":
                    limit = int.Parse(NextValue());
                    break;
                case "--output":
                    outputPath = NextValue();
                    break;
                case "--extensions":
                    extensions = NextValue()
                        .Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)
                        .Select(NormalizeExtension)
                        .Distinct(StringComparer.OrdinalIgnoreCase)
                        .ToList();
                    break;
                case "--help":
                case "-h":
                case "-?":
                    throw new ArgumentException(UsageText);
                default:
                    throw new ArgumentException($"Unknown option: {arg}");
            }
        }

        if (string.IsNullOrWhiteSpace(targetPath))
        {
            throw new ArgumentException("Missing target path.\n\n" + UsageText);
        }

        if (minLength < 1)
        {
            throw new ArgumentException("--min-length must be at least 1.");
        }

        return new Options
        {
            TargetPath = targetPath,
            BatchMode = batchMode,
            ReviewRoot = reviewRoot,
            Mode = mode,
            MinLength = minLength,
            Limit = limit,
            Extensions = extensions,
            OutputPath = outputPath
        };
    }

    public int Execute()
    {
        if (BatchMode)
        {
            return ExecuteBatch();
        }

        return ExecuteSingle();
    }

    private int ExecuteSingle()
    {
        var filePath = Path.GetFullPath(TargetPath);
        if (!File.Exists(filePath))
        {
            Console.Error.WriteLine($"File not found: {filePath}");
            return 1;
        }

        var data = File.ReadAllBytes(filePath);
        MarkdownExportContext? markdownContext = null;
        if (Mode == ExtractMode.Markdown && !string.IsNullOrWhiteSpace(OutputPath))
        {
            var outputPath = Path.GetFullPath(OutputPath);
            markdownContext = new MarkdownExportContext(null, InferReviewRoot(filePath, outputPath), outputPath);
        }

        if (!OutputBuilder.TryBuild(filePath, data, Mode, MinLength, Limit, markdownContext, out var output, out var error))
        {
            Console.Error.WriteLine(error);
            return 2;
        }

        if (string.IsNullOrWhiteSpace(OutputPath))
        {
            Console.Write(output);
        }
        else
        {
            var outputPath = Path.GetFullPath(OutputPath);
            Directory.CreateDirectory(Path.GetDirectoryName(outputPath)!);
            File.WriteAllText(outputPath, output, Encoding.UTF8);
        }

        return 0;
    }

    private static string? InferReviewRoot(string sourcePath, string outputPath)
    {
        var sourceSegments = Path.GetFullPath(sourcePath).Split(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar);
        var outputSegments = Path.GetFullPath(outputPath)
            .Replace(".md", string.Empty, StringComparison.OrdinalIgnoreCase)
            .Split(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar);

        var sourceIndex = sourceSegments.Length - 1;
        var outputIndex = outputSegments.Length - 1;
        while (sourceIndex >= 0 && outputIndex >= 0 && string.Equals(sourceSegments[sourceIndex], outputSegments[outputIndex], StringComparison.OrdinalIgnoreCase))
        {
            sourceIndex--;
            outputIndex--;
        }

        if (outputIndex <= 0 || outputIndex == outputSegments.Length - 1)
        {
            return null;
        }

        return string.Join(Path.DirectorySeparatorChar, outputSegments[..(outputIndex + 1)]);
    }

    private int ExecuteBatch()
    {
        var rootPath = Path.GetFullPath(TargetPath);
        if (!Directory.Exists(rootPath))
        {
            Console.Error.WriteLine($"Root path not found: {rootPath}");
            return 1;
        }

        var reviewRootPath = Path.IsPathRooted(ReviewRoot)
            ? Path.GetFullPath(ReviewRoot)
            : Path.GetFullPath(Path.Combine(rootPath, ReviewRoot));

        Directory.CreateDirectory(reviewRootPath);

        var extensionSet = new HashSet<string>(Extensions.Select(NormalizeExtension), StringComparer.OrdinalIgnoreCase);
        var report = BatchExporter.Export(rootPath, reviewRootPath, extensionSet, Mode, MinLength, Limit);
        Console.WriteLine($"Reviewed export complete for {report.RootPath} into {report.ReviewRootPath}; written: {report.WrittenCount}; failures: {report.Failures.Count}");
        return report.Failures.Count == 0 ? 0 : 2;
    }

    private static string NormalizeExtension(string extension) => extension.StartsWith('.') ? extension : "." + extension;

    private const string UsageText = """
Usage:
    blackbox-extract <file> [--mode review|summary|source|strings|markdown] [--limit 0] [--output path]
    blackbox-extract <root> --batch [--review-root review] [--mode review|summary|source|strings|markdown] [--extensions .odc,.ocf,.osf] [--limit 0]

Notes:
  --limit 0 means no truncation.
  Batch mode mirrors the source tree into the review folder and writes a manifest.
""";
}

static class BatchExporter
{
    public static BatchReport Export(string rootPath, string reviewRootPath, HashSet<string> extensions, ExtractMode mode, int minLength, int limit)
    {
        var report = new BatchReport(rootPath, reviewRootPath);
        var files = Directory.EnumerateFiles(rootPath, "*", SearchOption.AllDirectories)
            .Where(path => extensions.Contains(Path.GetExtension(path)))
            .Where(path => !IsGeneratedReviewPath(rootPath, reviewRootPath, path))
            .OrderBy(path => path, StringComparer.OrdinalIgnoreCase)
            .ToList();

        foreach (var filePath in files)
        {
            var relativePath = Path.GetRelativePath(rootPath, filePath);
            try
            {
                var outputPath = Path.Combine(reviewRootPath, relativePath + OutputBuilder.GetOutputExtension(mode));
                Directory.CreateDirectory(Path.GetDirectoryName(outputPath)!);
                var data = File.ReadAllBytes(filePath);
                MarkdownExportContext? markdownContext = null;
                if (mode == ExtractMode.Markdown)
                {
                    markdownContext = new MarkdownExportContext(rootPath, reviewRootPath, outputPath);
                }

                if (!OutputBuilder.TryBuild(filePath, data, mode, minLength, limit, markdownContext, out var output, out var error))
                {
                    report.Failures.Add((relativePath, error ?? "Unknown extraction failure."));
                    continue;
                }

                File.WriteAllText(outputPath, output, Encoding.UTF8);
                report.WrittenFiles.Add((relativePath, Path.GetRelativePath(reviewRootPath, outputPath)));
            }
            catch (Exception ex)
            {
                report.Failures.Add((relativePath, ex.Message));
            }
        }

        WriteManifest(report);
        return report;
    }

    private static bool IsGeneratedReviewPath(string rootPath, string reviewRootPath, string filePath)
    {
        if (filePath.StartsWith(reviewRootPath, StringComparison.OrdinalIgnoreCase))
        {
            return true;
        }

        var relativePath = Path.GetRelativePath(rootPath, filePath);
        var segments = relativePath.Split(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar);
        return segments.Any(static segment => Regex.IsMatch(segment, "^review($|[-_])", RegexOptions.IgnoreCase));
    }

    private static void WriteManifest(BatchReport report)
    {
        var manifestPath = Path.Combine(report.ReviewRootPath, "_manifest.txt");
        var lines = new List<string>
        {
            $"root: {report.RootPath}",
            $"review: {report.ReviewRootPath}",
            $"written: {report.WrittenCount}",
            $"failed: {report.Failures.Count}",
            string.Empty
        };

        foreach (var entry in report.WrittenFiles)
        {
            lines.Add($"{entry.SourceRelativePath} -> {entry.OutputRelativePath}");
        }

        if (report.Failures.Count > 0)
        {
            lines.Add(string.Empty);
            lines.Add("failures:");
            foreach (var failure in report.Failures)
            {
                lines.Add($"{failure.RelativePath} :: {failure.Message}");
            }
        }

        File.WriteAllLines(manifestPath, lines, Encoding.UTF8);
    }
}

sealed class BatchReport(string rootPath, string reviewRootPath)
{
    public string RootPath { get; } = rootPath;
    public string ReviewRootPath { get; } = reviewRootPath;
    public List<(string SourceRelativePath, string OutputRelativePath)> WrittenFiles { get; } = [];
    public List<(string RelativePath, string Message)> Failures { get; } = [];
    public int WrittenCount => WrittenFiles.Count;
}

static class Extractor
{
    private static readonly byte[] Magic = [(byte)'C', (byte)'D', (byte)'O', (byte)'o'];
    private static readonly Regex ModuleStartRegex = new(@"\bMODULE\s+([A-Za-z0-9_]+)\s*;", RegexOptions.IgnoreCase | RegexOptions.Compiled);
    private static readonly Regex ModuleEndRegex = new(@"\bEND\s+([A-Za-z0-9_]+)\s*\.", RegexOptions.IgnoreCase | RegexOptions.Compiled);
    private static readonly string[] OberonHints =
    [
        "MODULE",
        "IMPORT",
        "TYPE",
        "VAR",
        "CONST",
        "PROCEDURE",
        "BEGIN",
        "END",
        "RETURN",
        "POINTER TO",
        "RECORD",
        "ARRAY",
        "IF",
        "THEN",
        "ELSIF",
        "WHILE",
        "REPEAT",
        "UNTIL",
        "FOR",
        "CASE",
        "WITH",
        ":=",
        "(**",
        "*)"
    ];

    public static string BuildOutput(string resolvedPath, byte[] data, ExtractMode mode, int minLength, int limit)
    {
        var runs = ExtractAsciiRuns(data, minLength);
        var headerRuns = ExtractAsciiRuns(data.Take(Math.Min(512, data.Length)).ToArray(), 4);
        var moduleBody = TryReconstructModule(runs, minLength);
        var body = mode switch
        {
            ExtractMode.Summary => SourceLikeRuns(runs),
            ExtractMode.Source => SourceLikeRuns(runs),
            ExtractMode.Review => moduleBody ?? ReviewRuns(runs, minLength),
            _ => runs
        };

        var title = mode switch
        {
            ExtractMode.Summary => "sample source-like strings:",
            ExtractMode.Source => "source-like strings:",
            ExtractMode.Review => moduleBody is not null ? "reconstructed module:" : "review text:",
            _ => "printable strings:"
        };

        var sb = new StringBuilder();
        sb.AppendLine($"file: {resolvedPath}");
        sb.AppendLine($"size: {data.Length} bytes");
        sb.AppendLine($"magic: {(data.Length >= 4 ? Encoding.Latin1.GetString(data.AsSpan(0, 4)) : "<short file>")}");
        sb.AppendLine($"header_match: {(data.AsSpan().StartsWith(Magic) ? "yes" : "no")}");
        sb.AppendLine();
        sb.AppendLine("header strings:");
        foreach (var entry in headerRuns.Take(20))
        {
            sb.Append("  ").AppendLine(entry);
        }

        sb.AppendLine();
        sb.AppendLine(title);
        var effectiveLimit = limit <= 0 ? int.MaxValue : limit;
        var selected = body.Take(effectiveLimit).ToList();
        if (selected.Count == 0)
        {
            sb.AppendLine("  <none>");
        }
        else
        {
            foreach (var line in selected)
            {
                sb.AppendLine(line);
            }

            if (limit > 0 && body.Count > limit)
            {
                sb.AppendLine();
                sb.AppendLine($"... truncated after {limit} lines ...");
            }
        }

        return sb.ToString();
    }

    private static List<string> ExtractAsciiRuns(byte[] data, int minLength)
    {
        var runs = new List<string>();
        var buffer = new StringBuilder();
        foreach (var value in data)
        {
            if (IsPrintable(value))
            {
                buffer.Append((char)value);
            }
            else
            {
                Flush();
            }
        }

        Flush();
        return runs;

        void Flush()
        {
            if (buffer.Length >= minLength)
            {
                runs.Add(buffer.ToString());
            }

            buffer.Clear();
        }
    }

    private static List<string> SourceLikeRuns(List<string> runs) =>
        runs.Where(run => OberonHints.Any(hint => run.Contains(hint, StringComparison.OrdinalIgnoreCase))).ToList();

    private static List<string> ReviewRuns(List<string> runs, int minLength)
    {
        var results = new List<string>();
        foreach (var run in runs)
        {
            var trimmed = run.TrimEnd();
            if (trimmed.Length < minLength)
            {
                continue;
            }

            if (OberonHints.Any(hint => trimmed.Contains(hint, StringComparison.OrdinalIgnoreCase)))
            {
                results.Add(trimmed);
                continue;
            }

            var letterCount = trimmed.Count(char.IsLetter);
            if (letterCount < 6)
            {
                continue;
            }

            if (trimmed.Length >= 24 || trimmed.Contains(' ') || trimmed.Contains('.') || trimmed.Contains(';') || trimmed.Contains(':'))
            {
                results.Add(trimmed);
            }
        }

        return results;
    }

    private static List<string>? TryReconstructModule(List<string> runs, int minLength)
    {
        var lines = ExpandLines(runs);
        var startIndex = -1;
        var moduleName = string.Empty;

        for (var index = 0; index < lines.Count; index++)
        {
            var match = ModuleStartRegex.Match(lines[index]);
            if (!match.Success)
            {
                continue;
            }

            startIndex = index;
            moduleName = match.Groups[1].Value;
            break;
        }

        if (startIndex < 0)
        {
            return null;
        }

        var endIndex = -1;
        for (var index = startIndex; index < lines.Count; index++)
        {
            var match = ModuleEndRegex.Match(lines[index]);
            if (!match.Success)
            {
                continue;
            }

            if (string.Equals(match.Groups[1].Value, moduleName, StringComparison.OrdinalIgnoreCase))
            {
                endIndex = index;
                break;
            }
        }

        if (endIndex < 0)
        {
            return null;
        }

        var reconstructed = new List<string>();
        for (var index = Math.Max(0, startIndex - 1); index <= endIndex; index++)
        {
            var line = lines[index].TrimEnd();
            if (index < startIndex && !LooksLikeModulePreamble(line))
            {
                continue;
            }

            if (line.Length < minLength && !string.IsNullOrWhiteSpace(line))
            {
                continue;
            }

            reconstructed.Add(line);
        }

        while (reconstructed.Count > 0 && string.IsNullOrWhiteSpace(reconstructed[0]))
        {
            reconstructed.RemoveAt(0);
        }

        while (reconstructed.Count > 0 && string.IsNullOrWhiteSpace(reconstructed[^1]))
        {
            reconstructed.RemoveAt(reconstructed.Count - 1);
        }

        reconstructed = CleanReconstructedModule(reconstructed);
        return reconstructed.Count > 0 ? reconstructed : null;
    }

    private static List<string> ExpandLines(List<string> runs)
    {
        var lines = new List<string>();
        foreach (var run in runs)
        {
            using var reader = new StringReader(run);
            while (reader.ReadLine() is { } line)
            {
                lines.Add(line);
            }
        }

        return lines;
    }

    private static bool LooksLikeModulePreamble(string line)
    {
        var trimmed = line.Trim();
        if (trimmed.Length == 0)
        {
            return true;
        }

        return trimmed.StartsWith("(*", StringComparison.Ordinal)
            || trimmed.StartsWith("(**", StringComparison.Ordinal)
            || trimmed.StartsWith("-", StringComparison.Ordinal)
            || trimmed.Contains("YYYYMMDD", StringComparison.OrdinalIgnoreCase);
    }

    private static List<string> CleanReconstructedModule(List<string> lines)
    {
        var cleaned = new List<string>(lines.Count);
        var previousBlank = false;

        foreach (var line in lines)
        {
            var trimmed = line.Trim();
            if (LooksLikeEmbeddedPath(trimmed))
            {
                continue;
            }

            if (trimmed.Length == 0)
            {
                if (previousBlank)
                {
                    continue;
                }

                previousBlank = true;
                cleaned.Add(string.Empty);
                continue;
            }

            previousBlank = false;
            cleaned.Add(line);
        }

        return cleaned;
    }

    private static bool LooksLikeEmbeddedPath(string trimmed)
    {
        if (trimmed.Length == 0 || trimmed.Contains(' ') || trimmed.Contains('\t'))
        {
            return false;
        }

        if (!trimmed.Contains('/'))
        {
            return false;
        }

        return trimmed.All(ch => char.IsLetterOrDigit(ch) || ch is '/' or '-' or '_' or '.');
    }

    private static bool IsPrintable(byte value) => value is >= 32 and <= 126 or 9 or 10 or 13;
}