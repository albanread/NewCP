[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0, ParameterSetName = 'Single')]
    [string]$Path,

    [Parameter(Mandatory = $true, Position = 0, ParameterSetName = 'Batch')]
    [string]$RootPath,

    [Parameter(ParameterSetName = 'Batch')]
    [string]$ReviewRoot = 'review',

    [ValidateSet('summary', 'strings', 'source', 'review')]
    [string]$Mode = 'review',

    [int]$MinLength = 4,

    [int]$Limit = 200,

    [Parameter(ParameterSetName = 'Batch')]
    [string[]]$IncludeExtensions = @('.odc', '.ocf', '.osf'),

    [string]$Output
)

$magic = [byte[]](0x43, 0x44, 0x4F, 0x6F)
$latin1 = [System.Text.Encoding]::GetEncoding(28591)
$oberonHints = @(
    'MODULE',
    'IMPORT',
    'TYPE',
    'VAR',
    'CONST',
    'PROCEDURE',
    'BEGIN',
    'END',
    'RETURN',
    'POINTER TO',
    'RECORD',
    'ARRAY',
    'IF',
    'THEN',
    'ELSIF',
    'WHILE',
    'REPEAT',
    'UNTIL',
    'FOR',
    'CASE',
    'WITH',
    ':=',
    '(**',
    '*)'
)

function Test-PrintableByte {
    param([byte]$Byte)

    return (($Byte -ge 32 -and $Byte -le 126) -or $Byte -in 9, 10, 13)
}

function Get-AsciiRuns {
    param(
        [byte[]]$Data,
        [int]$MinimumLength
    )

    $runs = New-Object System.Collections.Generic.List[string]
    $buffer = New-Object System.Text.StringBuilder

    foreach ($byte in $Data) {
        if (Test-PrintableByte $byte) {
            [void]$buffer.Append([char]$byte)
            continue
        }

        if ($buffer.Length -ge $MinimumLength) {
            $runs.Add($buffer.ToString())
        }

        [void]$buffer.Clear()
    }

    if ($buffer.Length -ge $MinimumLength) {
        $runs.Add($buffer.ToString())
    }

    return $runs
}

function Get-SourceLikeRuns {
    param([System.Collections.Generic.List[string]]$Runs)

    $filtered = New-Object System.Collections.Generic.List[string]
    foreach ($run in $Runs) {
        $upper = $run.ToUpperInvariant()
        foreach ($hint in $oberonHints) {
            if ($upper.Contains($hint)) {
                $filtered.Add($run)
                break
            }
        }
    }
    return $filtered
}

function Test-ReviewCandidate {
    param([string]$Run)

    $trimmed = $Run.Trim()
    if ($trimmed.Length -lt $MinLength) {
        return $false
    }

    $upper = $trimmed.ToUpperInvariant()
    foreach ($hint in $oberonHints) {
        if ($upper.Contains($hint)) {
            return $true
        }
    }

    $letterCount = 0
    foreach ($char in $trimmed.ToCharArray()) {
        if ([char]::IsLetter($char)) {
            $letterCount++
        }
    }

    if ($letterCount -lt 6) {
        return $false
    }

    if ($trimmed.Length -ge 24) {
        return $true
    }

    return ($trimmed.Contains(' ') -or $trimmed.Contains('.') -or $trimmed.Contains(';') -or $trimmed.Contains(':'))
}

function Get-ReviewRuns {
    param([System.Collections.Generic.List[string]]$Runs)

    $filtered = New-Object System.Collections.Generic.List[string]
    foreach ($run in $Runs) {
        if (Test-ReviewCandidate $run) {
            $filtered.Add($run.TrimEnd())
        }
    }
    return $filtered
}

function Get-BodyRuns {
    param(
        [System.Collections.Generic.List[string]]$Runs,
        [string]$SelectedMode
    )

    switch ($SelectedMode) {
        'summary' {
            return @{ Title = 'sample source-like strings:'; Runs = (Get-SourceLikeRuns -Runs $Runs) }
        }
        'source' {
            return @{ Title = 'source-like strings:'; Runs = (Get-SourceLikeRuns -Runs $Runs) }
        }
        'review' {
            return @{ Title = 'review text:'; Runs = (Get-ReviewRuns -Runs $Runs) }
        }
        default {
            return @{ Title = 'printable strings:'; Runs = $Runs }
        }
    }
}

function Build-OutputText {
    param(
        [string]$ResolvedPath,
        [byte[]]$Data,
        [string]$SelectedMode,
        [int]$SelectedMinLength,
        [int]$SelectedLimit
    )

    $runs = Get-AsciiRuns -Data $Data -MinimumLength $SelectedMinLength
    $headerCount = [Math]::Min(512, $Data.Length)
    if ($headerCount -gt 0) {
        $headerData = $Data[0..($headerCount - 1)]
        $headerRuns = Get-AsciiRuns -Data $headerData -MinimumLength 4
    } else {
        $headerRuns = New-Object System.Collections.Generic.List[string]
    }
    $bodyInfo = Get-BodyRuns -Runs $runs -SelectedMode $SelectedMode
    $bodyRuns = $bodyInfo.Runs
    $effectiveLimit = if ($SelectedLimit -le 0) { $bodyRuns.Count } else { $SelectedLimit }

    $lines = New-Object System.Collections.Generic.List[string]
    $lines.Add("file: $ResolvedPath")
    $lines.Add("size: $($Data.Length) bytes")
    if ($Data.Length -ge 4) {
        $lines.Add("magic: " + $latin1.GetString($Data[0..3]))
    } else {
        $lines.Add('magic: <short file>')
    }
    $matchesMagic = $Data.Length -ge 4 -and ($latin1.GetString($Data[0..3]) -eq 'CDOo')
    $lines.Add("header_match: $(if ($matchesMagic) { 'yes' } else { 'no' })")
    $lines.Add('')
    $lines.Add('header strings:')

    foreach ($entry in ($headerRuns | Select-Object -First 20)) {
        $lines.Add("  $entry")
    }

    $lines.Add('')
    $lines.Add($bodyInfo.Title)

    $selectedRuns = $bodyRuns | Select-Object -First $effectiveLimit
    if (-not $selectedRuns) {
        $lines.Add('  <none>')
    } else {
        foreach ($entry in $selectedRuns) {
            $lines.Add($entry)
        }
        if (($SelectedLimit -gt 0) -and ($bodyRuns.Count -gt $SelectedLimit)) {
            $lines.Add('')
            $lines.Add("... truncated after $SelectedLimit lines ...")
        }
    }

    return ($lines -join [Environment]::NewLine) + [Environment]::NewLine
}

function Get-ReviewOutputPath {
    param(
        [string]$Root,
        [string]$Review,
        [string]$FilePath
    )

    $resolvedRoot = [System.IO.Path]::GetFullPath($Root)
    $resolvedFile = [System.IO.Path]::GetFullPath($FilePath)
    $relativePath = $resolvedFile.Substring($resolvedRoot.Length).TrimStart([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
    return (Join-Path $Review ($relativePath + '.txt'))
}

function Resolve-ReviewRootPath {
    param(
        [string]$Root,
        [string]$Review
    )

    if ([System.IO.Path]::IsPathRooted($Review)) {
        return [System.IO.Path]::GetFullPath($Review)
    }

    return [System.IO.Path]::GetFullPath((Join-Path $Root $Review))
}

function Test-IsGeneratedReviewPath {
    param(
        [string]$Root,
        [string]$FilePath,
        [string]$ResolvedReview
    )

    if ($FilePath.StartsWith($ResolvedReview, [System.StringComparison]::OrdinalIgnoreCase)) {
        return $true
    }

    $relativePath = $FilePath.Substring($Root.Length).TrimStart([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
    if ([string]::IsNullOrEmpty($relativePath)) {
        return $false
    }

    foreach ($segment in ($relativePath -split '[\\/]')) {
        if ($segment -match '^review($|[-_])') {
            return $true
        }
    }

    return $false
}

function Get-ManifestName {
    param([string]$Root)

    $leaf = Split-Path -Leaf $Root
    if ([string]::IsNullOrWhiteSpace($leaf)) {
        $leaf = 'root'
    }
    $safeLeaf = ($leaf -replace '[^A-Za-z0-9._-]', '_')
    return "_manifest_$safeLeaf.txt"
}

function Export-ReviewTree {
    param(
        [string]$Root,
        [string]$Review,
        [string]$SelectedMode,
        [int]$SelectedMinLength,
        [int]$SelectedLimit,
        [string[]]$Extensions
    )

    $resolvedRoot = (Resolve-Path -LiteralPath $Root).Path
    $resolvedReview = Resolve-ReviewRootPath -Root $resolvedRoot -Review $Review
    [System.IO.Directory]::CreateDirectory($resolvedReview) | Out-Null

    $allowedExtensions = @{}
    foreach ($extension in $Extensions) {
        $normalizedExtension = if ($extension.StartsWith('.')) { $extension.ToLowerInvariant() } else { '.' + $extension.ToLowerInvariant() }
        $allowedExtensions[$normalizedExtension] = $true
    }

    $files = Get-ChildItem -LiteralPath $resolvedRoot -Recurse -File |
        Where-Object {
            -not (Test-IsGeneratedReviewPath -Root $resolvedRoot -FilePath $_.FullName -ResolvedReview $resolvedReview) -and
            $allowedExtensions.ContainsKey($_.Extension.ToLowerInvariant())
        }

    $manifest = New-Object System.Collections.Generic.List[string]
    $failures = New-Object System.Collections.Generic.List[string]
    $manifest.Add("root: $resolvedRoot")
    $manifest.Add("review: $resolvedReview")
    $manifest.Add("mode: $SelectedMode")
    $manifest.Add("files: $($files.Count)")
    $manifest.Add('')

    foreach ($file in $files) {
        try {
            $outputPath = Get-ReviewOutputPath -Root $resolvedRoot -Review $resolvedReview -FilePath $file.FullName
            $outputDirectory = Split-Path -Parent $outputPath
            [System.IO.Directory]::CreateDirectory($outputDirectory) | Out-Null

            $data = [System.IO.File]::ReadAllBytes($file.FullName)
            $outputText = Build-OutputText -ResolvedPath $file.FullName -Data $data -SelectedMode $SelectedMode -SelectedMinLength $SelectedMinLength -SelectedLimit $SelectedLimit
            [System.IO.File]::WriteAllText($outputPath, $outputText)
            $manifest.Add(($file.FullName.Substring($resolvedRoot.Length).TrimStart([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)) + ' -> ' + $outputPath.Substring($resolvedReview.Length).TrimStart([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar))
        } catch {
            $relativePath = $file.FullName.Substring($resolvedRoot.Length).TrimStart([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
            $failures.Add($relativePath + ' :: ' + $_.Exception.Message)
        }
    }

    $manifest.Insert(4, "written: $($manifest.Count - 5)")
    $manifest.Insert(5, "failed: $($failures.Count)")
    if ($failures.Count -gt 0) {
        $manifest.Add('')
        $manifest.Add('failures:')
        foreach ($failure in $failures) {
            $manifest.Add($failure)
        }
    }

    [System.IO.File]::WriteAllText((Join-Path $resolvedReview (Get-ManifestName -Root $resolvedRoot)), (($manifest -join [Environment]::NewLine) + [Environment]::NewLine))
    return [pscustomobject]@{
        Root = $resolvedRoot
        Review = $resolvedReview
        Count = $files.Count - $failures.Count
        Failed = $failures.Count
    }
}

if ($PSCmdlet.ParameterSetName -eq 'Batch') {
    if (-not (Test-Path -LiteralPath $RootPath)) {
        throw "Root path not found: $RootPath"
    }
    $result = Export-ReviewTree -Root $RootPath -Review $ReviewRoot -SelectedMode $Mode -SelectedMinLength $MinLength -SelectedLimit $Limit -Extensions $IncludeExtensions
    "Reviewed export complete for $($result.Root) into $($result.Review); failures: $($result.Failed)"
} else {
    if (-not (Test-Path -LiteralPath $Path)) {
        throw "File not found: $Path"
    }

    $resolvedPath = (Resolve-Path -LiteralPath $Path).Path
    $data = [System.IO.File]::ReadAllBytes($resolvedPath)
    $outputText = Build-OutputText -ResolvedPath $resolvedPath -Data $data -SelectedMode $Mode -SelectedMinLength $MinLength -SelectedLimit $Limit

    if ($Output) {
        [System.IO.File]::WriteAllText($Output, $outputText)
    } else {
        $outputText
    }
}