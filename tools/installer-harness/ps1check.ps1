# The Windows half, measured rather than read: the file has to parse, and the
# frames it would draw have to be the frames install.sh draws. A logo whose
# geometry is right on one installer and wrong on the other is exactly what two
# spellings of one shape are supposed to prevent.

$ErrorActionPreference = 'Stop'
$installer = Join-Path $PSScriptRoot 'install.ps1'

# --- it parses -------------------------------------------------------------

$tokens = $null
$errors = $null
[System.Management.Automation.Language.Parser]::ParseFile(
    $installer, [ref]$tokens, [ref]$errors) | Out-Null
if ($errors.Count -gt 0) {
    foreach ($e in $errors) { "PARSE $($e.Extent.StartLineNumber): $($e.Message)" }
    exit 1
}
'install.ps1 parses'

# --- the drawing functions, lifted out and run -----------------------------
#
# The file installs when it is run, so the three functions the logo is made of
# are taken out of the parsed tree and defined here on their own. Reading them
# out of the AST rather than copying them is what makes this a test of the file
# and not of a copy that can drift from it.

$ast = [System.Management.Automation.Language.Parser]::ParseFile(
    $installer, [ref]$tokens, [ref]$errors)

$wanted = 'Get-LogoBand', 'Get-LogoFrame', 'Test-GlyphSurvives', 'Say', 'Ok'
$found = @{}
foreach ($fn in $ast.FindAll({ $args[0] -is
        [System.Management.Automation.Language.FunctionDefinitionAst] }, $true)) {
    if ($wanted -contains $fn.Name) { $found[$fn.Name] = $fn.Extent.Text }
}
foreach ($w in $wanted) {
    if (-not $found.ContainsKey($w)) { "MISSING function $w"; exit 1 }
}

$LogoWidth = 24
$LogoPad11 = '           '
$UiDim = ''
foreach ($w in $wanted) { Invoke-Expression $found[$w] }

# --- the frames ------------------------------------------------------------

function Render {
    param($Base, $Legs, $Stem, $Loop, $Soft, $Name)
    $cell = [string][char]0x2588
    $frame = Get-LogoFrame $Base $Legs $Stem $Loop $Soft $Name `
        ($cell * 2) ($cell * 4) ($cell * 14)
    $out = @()
    foreach ($row in $frame) {
        $line = ''
        foreach ($seg in $row) { $line += $seg.T }
        $out += $line.TrimEnd()
    }
    return , $out
}

$final = Render 1 4 5 1 'none' $true
# Parenthesised one by one: in PowerShell the comma binds tighter than the
# plus, so an unbracketed list of concatenations is one long string and not
# twelve rows.
$expected = @(
    ('          ' + ([string][char]0x2588 * 4)),
    ('        ' + ([string][char]0x2588 * 2) + '    ' + ([string][char]0x2588 * 2)),
    ('        ' + ([string][char]0x2588 * 2) + '    ' + ([string][char]0x2588 * 2)),
    ('          ' + ([string][char]0x2588 * 4)),
    ('           ' + ([string][char]0x2588 * 2)),
    ('   ' + ([string][char]0x2588 * 4) + '    ' + ([string][char]0x2588 * 2) + '    ' + ([string][char]0x2588 * 4)),
    ('   ' + ([string][char]0x2588 * 2) + '      ' + ([string][char]0x2588 * 2) + '      ' + ([string][char]0x2588 * 2)),
    ('   ' + ([string][char]0x2588 * 2) + '      ' + ([string][char]0x2588 * 2) + '      ' + ([string][char]0x2588 * 2)),
    ('   ' + ([string][char]0x2588 * 2) + '      ' + ([string][char]0x2588 * 2) + '      ' + ([string][char]0x2588 * 2)),
    ('     ' + ([string][char]0x2588 * 14)),
    '',
    '          ank'
)

$fails = @()
if ($final.Count -ne 12) { $fails += "the frame is $($final.Count) rows and twelve is the contract" }
for ($i = 0; $i -lt $expected.Count; $i++) {
    if ($final[$i] -ne $expected[$i]) {
        $fails += "row $($i + 1): got '$($final[$i])' wanted '$($expected[$i])'"
    }
}

# Every row of every frame fits the width the console was measured against.
$states = @(
    @(1, 0, 0, 0, 'base'), @(1, 1, 0, 0, 'none'), @(1, 4, 0, 0, 'none'),
    @(1, 4, 1, 0, 'none'), @(1, 4, 5, 0, 'none'), @(1, 4, 5, 1, 'loop'),
    @(1, 4, 5, 1, 'all')
)
foreach ($s in $states) {
    foreach ($line in (Render $s[0] $s[1] $s[2] $s[3] $s[4] $false)) {
        if ($line.Length -gt $LogoWidth) {
            $fails += "a row of state $($s[4]) is $($line.Length) wide, over $LogoWidth"
        }
    }
}

# An empty state draws nothing at all, which is what lets the redraw shrink.
foreach ($line in (Render 0 0 0 0 'none' $false)) {
    if ($line -ne '') { $fails += "the empty state drew '$line'" }
}

# The glyph test answers for the encodings it has to answer for.
$saved = [Console]::OutputEncoding
try {
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    if (-not (Test-GlyphSurvives 0x2588)) { $fails += 'UTF-8 was told it cannot carry the block' }
    if (-not (Test-GlyphSurvives 0x2713)) { $fails += 'UTF-8 was told it cannot carry the tick' }

    [System.Text.Encoding]::RegisterProvider([System.Text.CodePagesEncodingProvider]::Instance)
    [Console]::OutputEncoding = [System.Text.Encoding]::GetEncoding(437)
    if (-not (Test-GlyphSurvives 0x2588)) { $fails += 'codepage 437 was told it cannot carry the block, and it carries it at 0xDB' }
    if (Test-GlyphSurvives 0x2713) { $fails += 'codepage 437 was told it carries the tick; it best-fits it to a square root sign' }
    # The ANSI codepage best-fits the block to a broken bar, which is the case
    # a question-mark test waves through and the round trip refuses.
    [Console]::OutputEncoding = [System.Text.Encoding]::GetEncoding(1252)
    if (Test-GlyphSurvives 0x2588) { $fails += 'codepage 1252 was told it carries the block; it best-fits it to a broken bar' }
} finally {
    [Console]::OutputEncoding = $saved
}

# --- what a transcript reads ----------------------------------------------
#
# ADR-5fbd99bf6fd5 promises a caller with no console the installer it sees
# today, to the byte. Every sequence and every marker is empty until Enable-Ui
# runs, so with them empty a marked step has to come out as the bare sentence
# it used to be -- one line, no indent, no tick.
$UiPad = ''
$UiTick = ''
$UiGreen = ''
$UiCyan = ''

# Write-Host emits one information record per call, and Out-String would put a
# newline between them regardless of -NoNewline. So the console line is
# reconstructed from the records themselves, honouring the flag each carries --
# otherwise this would measure the capture and not the installer.
function Capture {
    param([scriptblock]$Block)
    $text = ''
    foreach ($rec in (& $Block 6>&1)) {
        $msg = $rec.MessageData
        $text += [string]$msg.Message
        if (-not $msg.NoNewLine) { $text += "`n" }
    }
    return $text.TrimEnd("`n")
}

$plain = Capture { Ok 'checksum ok  abc123' }
if ($plain -ne 'checksum ok  abc123') {
    $fails += "an unenabled Ok wrote '$plain' where the bare sentence was expected"
}

$plain = Capture { Say 'installed  C:\x\ank.exe' }
if ($plain -ne 'installed  C:\x\ank.exe') {
    $fails += "an unenabled Say wrote '$plain'"
}

# And with them enabled it is marked, indented, and still says what it says.
$UiPad = '  '
$UiTick = "$([char]0x2713) "
$UiGreen = 'Green'
$marked = Capture { Ok 'checksum ok  abc123' }
if ($marked -ne "  $([char]0x2713) checksum ok  abc123") {
    $fails += "an enabled Ok wrote '$marked'"
}

'-' * 60
if ($fails.Count -gt 0) {
    foreach ($f in $fails) { "FAIL $f" }
    exit 1
}
'the frame is twelve rows, matches install.sh row for row, never exceeds'
"$LogoWidth columns, draws nothing when empty, and the glyph test answers"
'correctly for UTF-8 and for codepage 437. An unenabled step is the bare'
'sentence it always was, and an enabled one is marked and indented.'
