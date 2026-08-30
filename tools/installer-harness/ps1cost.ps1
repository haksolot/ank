# What the animation costs a console, counted rather than timed.
#
# A wall in milliseconds measures this machine, and this machine is not a
# Windows console. What is portable is the number of writes: conhost turns each
# Write-Host that changes an attribute into a console API call, and that count
# is the same on every machine that runs this file.

$ErrorActionPreference = 'Stop'
$installer = Join-Path $PSScriptRoot 'install.ps1'

$tokens = $null
$errors = $null
$ast = [System.Management.Automation.Language.Parser]::ParseFile(
    $installer, [ref]$tokens, [ref]$errors)

$wanted = 'Get-LogoBand', 'Get-LogoFrame', 'Write-LogoRow'
$found = @{}
foreach ($fn in $ast.FindAll({ $args[0] -is
        [System.Management.Automation.Language.FunctionDefinitionAst] }, $true)) {
    if ($wanted -contains $fn.Name) { $found[$fn.Name] = $fn.Extent.Text }
}

$LogoWidth = 24
$LogoPad11 = '           '
$UiDim = 'DarkGray'
foreach ($w in $wanted) { Invoke-Expression $found[$w] }

# Say, replaced by a counter. Write-LogoRow reaches the console through it and
# through nothing else, so what this counts is exactly what a console is asked
# to do.
$script:calls = 0
$script:colourChanges = 0
$script:lastColour = ''
function Say {
    param([string]$Line = '', [switch]$NoNewline, [string]$Color = '')
    $script:calls++
    if ($Color -ne $script:lastColour) {
        $script:colourChanges++
        $script:lastColour = $Color
    }
}

# The timeline install.ps1 holds, read out of the file rather than copied, so
# this counts the frames that are actually drawn.
$showLogo = ($ast.FindAll({ $args[0] -is
        [System.Management.Automation.Language.FunctionDefinitionAst] -and
        $args[0].Name -eq 'Show-Logo' }, $true))[0].Extent.Text
$m = [regex]::Matches($showLogo, '@\((\d+), (\d+), (\d+), (\d+), ''(\w+)'', \$(\w+), (\d+)\)')
if ($m.Count -lt 10) { "could not read the timeline out of Show-Logo"; exit 1 }

$cell = [string][char]0x2588
foreach ($step in $m) {
    $frame = Get-LogoFrame ([int]$step.Groups[1].Value) ([int]$step.Groups[2].Value) `
        ([int]$step.Groups[3].Value) ([int]$step.Groups[4].Value) `
        $step.Groups[5].Value ($step.Groups[6].Value -eq 'true') `
        ($cell * 2) ($cell * 4) ($cell * 14)
    foreach ($row in $frame) { Write-LogoRow $row }
}

"frames drawn:      $($m.Count)"
"writes to console: $($script:calls)"
"attribute changes: $($script:colourChanges)"
"writes per frame:  $([math]::Round($script:calls / $m.Count, 1))"
