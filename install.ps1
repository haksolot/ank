<#
Install ank from a GitHub release.

  irm https://raw.githubusercontent.com/haksolot/ank/main/install.ps1 | iex

  & ([scriptblock]::Create((irm https://raw.githubusercontent.com/haksolot/ank/main/install.ps1))) -Version v0.2.0

The Windows counterpart of install.sh, and the same contract: fetch the archive
the release published, verify it against the .sha256 published beside it, unpack
it, and never end in silence. What differs is only what Windows spells
differently.

One executable lands: ank.exe. The protocol surface and the watcher are verbs
of it (ADR-1ea31c2f3c5a), so there is no second file for this script to place
and no way for one to fail to arrive.

-Version reaches releases published before that was true, whose archive carries
a second executable beside ank.exe. Nothing here asks: the archive is unpacked,
ank.exe is taken out of it, and whatever else the directory holds goes with the
temporary directory. So an old release installs the one file this script
promises, by the same code path a new one does.

Windows PowerShell 5.1 is the floor, because that is what a clean Windows ships
and this channel exists for the machine that has nothing installed yet. Two
consequences run through the file: TLS 1.2 is turned on explicitly, since 5.1
still defaults to protocols GitHub refuses, and every cmdlet used here is one
5.1 carries.

Exit codes, so a caller can branch on the failure rather than on the message:

  1  usage, or a directory that cannot be written
  2  unsupported platform
  3  the download failed, or the release does not carry this archive
  4  the checksum did not match the one the release published
  5  a runtime this script needs is missing

-NoWelcome, or ANK_NO_WELCOME in the environment, turns off everything this
script draws for a human and everything it asks one, and leaves only the lines
a machine reads. It is absent from the list above on purpose: the welcome is
drawn before the first request goes out and the offer comes after the binary is
on disk and verified, so neither is on the path to any of them -- a flag able
to change one of these five codes would be a flag that made the install depend
on it.

They are the codes install.sh returns for the same failures, and they reach a
caller who runs this as a file. The one-liner above runs it through `iex`, where
`exit` would close the window the caller is typing in -- so under `iex` a
failure throws instead, after printing the same diagnosis. That branch is the
whole reason $PSCommandPath is read below: it is empty exactly when this text is
not being run as a file.
#>

param(
    [string]$Version,
    [string]$Dir,
    [switch]$NoWelcome,
    [switch]$Help
)

# No Set-StrictMode here, and it is not an oversight. Under `iex` this text runs
# in the caller's own session, where a strict mode set by an installer would
# outlive it and change how their next commands behave -- and unlike the two
# settings restored in the finally below, there is nothing to read it back from.

$Repo = 'haksolot/ank'
$RawUrl = "https://raw.githubusercontent.com/$Repo/main/install.ps1"
$ReleasesUrl = "https://github.com/$Repo/releases"
$DefaultBaseUrl = "$ReleasesUrl/download"

# The one target release.yml builds for Windows. Written once and read by both
# the refusal and the help text: a list that says one thing when it refuses and
# another when it is asked is worse than no list at all.
$WindowsTarget = 'x86_64-pc-windows-msvc'

# Empty exactly when this text is not running as a file -- `irm | iex`, or a
# scriptblock built from it. Probed rather than assumed, including from inside
# another script, which is the case a CI step is.
$RunningAsFile = -not [string]::IsNullOrEmpty($PSCommandPath)

# ---------------------------------------------------------------------------
# Output
# ---------------------------------------------------------------------------

# Write-Host and not Write-Output, for the reason install.sh sends everything to
# stderr: under `iex` this runs in the caller's own session, and a diagnosis
# written to the pipeline would land in whatever they were assembling.
function Say {
    param([string]$Line = '', [switch]$NoNewline)
    if ($NoNewline) { Write-Host -NoNewline $Line } else { Write-Host $Line }
}

# `Fail <code> <lines>`: the code carries the kind of failure, the lines carry
# what to do next. Nothing here ever ends in silence -- that is the one thing an
# install script cannot do, since the caller is otherwise left with no binary
# and no idea why.
function Fail {
    param([int]$Code, [string[]]$Lines)

    Say "ank: $($Lines[0])"
    for ($i = 1; $i -lt $Lines.Count; $i++) { Say $Lines[$i] }

    if ($RunningAsFile) { exit $Code }
    # Under `iex` the caller is at their own prompt: `exit` would take the
    # session with it, which is a worse outcome than the failed install.
    throw "ank: install failed (code $Code)"
}

function Show-Usage {
    Say 'install ank from a GitHub release'
    Say ''
    Say 'One executable lands in the install directory: ank.exe. The protocol'
    Say 'surface and the watcher are verbs of it -- ank mcp, ank watch -- so'
    Say 'there is nothing further to fetch and nothing further to configure a'
    Say 'client against.'
    Say ''
    Say 'usage:'
    Say "  irm $RawUrl | iex"
    Say ''
    Say '  with arguments, since a pipe into iex cannot carry them:'
    Say "  & ([scriptblock]::Create((irm $RawUrl))) -Version v0.2.0"
    Say ''
    Say 'options:'
    Say '  -Version <version>  install this release instead of the latest one;'
    Say '                      "v0.2.0" and "0.2.0" both work'
    Say '  -Dir <path>         install into <path> instead of'
    Say '                      $env:LOCALAPPDATA\Programs\ank'
    Say '  -NoWelcome          draw nothing and ask nothing; install exactly what an'
    Say '                      interactive run that declined every offer installs'
    Say '  -Help               print this and exit'
    Say ''
    Say 'environment:'
    Say '  ANK_VERSION         same as -Version'
    Say '  ANK_INSTALL_DIR     same as -Dir'
    Say '  ANK_NO_WELCOME      same as -NoWelcome, for a caller that pipes this script'
    Say '                      into iex and cannot pass a switch to it'
    Say '  ANK_BASE_URL        where the archives are fetched from, for a mirror or a'
    Say '                      staged release; requires -Version, since only GitHub'
    Say '                      can be asked which release is the latest'
    Say ''
    Say 'platforms:'
    Say "  Windows x64         $WindowsTarget"
    Say ''
    Say '  Linux and macOS are installed by install.sh:'
    Say "    curl -fsSL https://raw.githubusercontent.com/$Repo/main/install.sh | sh"
    Say ''
    Say 'the two questions:'
    Say '  With a console attached, once ank is installed and verified, this asks two'
    Say '  things and nothing else. The first offers to run:'
    Say "    npx skills add $Repo"
    Say '  which teaches an agent how to use ank. The second offers to print three'
    Say '  prompts that adopt ank in a repository that already has code and no .ank;'
    Say '  they are in docs/getting-started.md too, and printing them writes nothing'
    Say '  anywhere.'
    Say '  Enter accepts each, declining either does nothing at all, and nothing'
    Say '  either does can change any of the codes below.'
    Say ''
    Say 'exit codes:'
    Say '  1 usage   2 unsupported platform   3 download   4 checksum   5 missing runtime'
}

# ---------------------------------------------------------------------------
# Welcome
# ---------------------------------------------------------------------------

# The logo, at half the resolution of assets/ank.svg and drawn in ASCII: the
# same twelve lines install.sh holds, because it is the same logo and two
# spellings of it would drift. That file is the reference for the shape and
# nothing here reads it -- an installer that fetches a logo before it fetches
# the binary is an installer with a second way to fail before doing anything
# useful, so the frames are bytes in this file.
#
# ASCII and not U+2588, because the console this lands in is not always UTF-8:
# a Windows PowerShell 5.1 in conhost under codepage 437 renders a block
# character as a question mark, and a logo that arrives as question marks is
# worse than no logo at all.
$LogoArt = @(
    '          ####',
    '        ##    ##',
    '        ##    ##',
    '          ####',
    '          ####',
    '  ######  ####  ######',
    '  ####    ####    ####',
    '  ####    ####    ####',
    '  ####    ####    ####',
    '    ################',
    '',
    '          ank'
)
$LogoWidth = 24

# The cursor is moved through $Host.UI.RawUI and not with an escape sequence,
# and that is the Windows half of this decision rather than a preference.
# Windows PowerShell 5.1 in conhost does not turn on virtual terminal
# processing for its own output, so an ESC[12A written there is printed as
# glyphs instead of obeyed: the animation would become exactly the mess it
# exists to avoid, on precisely the shell this file claims as its floor. RawUI
# is the interface both hosts implement, and it emits no escape sequence at
# all, on any run.
function Show-Logo {
    $ui = $Host.UI.RawUI

    # A window narrower than the art wraps every line, and a block that is
    # taller than twelve rows is a block the redraw moves back over the middle
    # of. Nothing is written before this is known.
    if ($ui.WindowSize.Width -lt $LogoWidth) { return }

    # The blank lines are written before the position is read, so that a window
    # with the prompt at its bottom does its scrolling now: a coordinate saved
    # while the buffer is still moving points at the wrong row for every frame
    # after it.
    for ($i = 0; $i -lt $LogoArt.Count; $i++) { Say '' }

    $top = $ui.CursorPosition.Y - $LogoArt.Count
    if ($top -lt 0) { return }
    $origin = New-Object System.Management.Automation.Host.Coordinates 0, $top

    $savedCursor = $true
    try {
        $savedCursor = [Console]::CursorVisible
        [Console]::CursorVisible = $false
    } catch {
        # A host that will not say whether its cursor is visible still draws.
    }

    try {
        # Bottom up: the base, then the stem, then the loop, which is the order
        # the shape is built in and the order that leaves the whole of it
        # standing at the end. Frame 11 adds the name, and that is the beat the
        # animation exists for: it costs the time it takes to read the name of
        # the tool and not a second more.
        for ($frame = 1; $frame -le 11; $frame++) {
            $ui.CursorPosition = $origin
            for ($line = 1; $line -le $LogoArt.Count; $line++) {
                $text = ''
                if ($line -le 10 -and $line -ge (11 - $frame)) {
                    $text = $LogoArt[$line - 1]
                } elseif ($line -eq 12 -and $frame -eq 11) {
                    $text = $LogoArt[11]
                }
                # Padded rather than erased: there is no clear-to-end-of-line
                # without an escape sequence, and writing the spaces says the
                # same thing in the alphabet this function is restricted to.
                Say $text.PadRight($LogoWidth)
            }
            if ($frame -lt 11) { Start-Sleep -Milliseconds 60 }
        }
    } finally {
        try { [Console]::CursorVisible = $savedCursor } catch { }
    }

    # One blank line under the block, so what the install says next does not
    # start on the line the name ends on.
    Say ''
}

# ADR-5fbd99bf6fd5 read as an absence: where no human is looking, this script
# draws nothing at all and asks nothing at all.
#
# Both streams are tested and not only one. `irm ... | iex` leaves stdout and
# stderr on the console, which is the case that must animate; redirecting
# either of them into a file is the case that must not, since a transcript full
# of padding and repositioning is the noise this exists to avoid. The host is
# asked last, and by trying rather than by name: a host with no console answers
# the two properties above happily and then throws on the first cursor it is
# asked to place.
#
# The logo and the offer read this same answer, which is what makes -NoWelcome
# and an interactive run that declined everything leave the same machine
# behind: one predicate, so there is no second gate to disagree with this one.
# The width belongs to the block of art alone and is asked inside Show-Logo,
# since a console too narrow to hold the logo is still a console with a person
# in front of it.
function Test-HumanAtTerminal {
    if ($NoWelcome) { return $false }
    if ($env:ANK_NO_WELCOME) { return $false }
    # A runner sets this, and a runner is a machine with no human at it.
    if ($env:CI) { return $false }
    try {
        if ([Console]::IsOutputRedirected) { return $false }
        if ([Console]::IsErrorRedirected) { return $false }
        $null = $Host.UI.RawUI.CursorPosition
    } catch {
        return $false
    }
    return $true
}

# ---------------------------------------------------------------------------
# The skills
# ---------------------------------------------------------------------------

# ADR-5fbd99bf6fd5's offer, and the last thing this script does. Everything
# else has already happened by then: the binary is on disk, verified, reported,
# and the PATH advice given. That ordering is the decision rather than a layout
# -- an installation that stops to ask something is an installation that can be
# abandoned half-done, and half-done is the worst state for a tool whose next
# action is `ank context`.
#
# `npx skills add <owner>/ank` is what skill/SKILL.md already teaches, and it
# serves every agent the skills CLI knows about rather than one. An installer
# that learned where each of them keeps its skills is an installer that goes
# stale silently, so this one hands that work to the tool whose job it is.
#
# Nothing in here is allowed to reach the caller as a failure. The call site
# wraps it, and the exit code is stamped after it returns.
function Invoke-SkillOffer {
    if (-not (Test-HumanAtTerminal)) { return }

    Say ''
    Say 'The skills teach an agent how to use ank: the contract, and one policy'
    Say 'per activity. They install through the skills CLI, which puts them where'
    Say 'each agent looks.'
    Say ''
    Say "  npx skills add $Repo"
    Say ''

    # [Console]::ReadLine and not Read-Host, which is where Windows differs
    # from the /dev/tty install.sh opens and why the two files spell this step
    # differently. `irm ... | iex` runs this text through a pipeline in the
    # caller's own session: Console.In is the console and never that pipeline,
    # so a question asked here cannot consume what they piped. $null is end of
    # input -- nothing was typed, so nothing is assumed.
    Say -NoNewline 'Install them now? [Y/n] '
    $answer = $null
    try { $answer = [Console]::ReadLine() } catch { $answer = $null }
    if ($null -eq $answer) {
        Say ''
        return
    }

    # Enter is yes and everything unrecognised is no, in that order: a default
    # the criterion names, and a decline for anything else because asking twice
    # is asking twice.
    $reply = $answer.Trim()
    if ($reply -ne '' -and $reply -notmatch '^(y|yes)$') { return }

    if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
        Say ''
        Say "  npx skills add $Repo"
        Say 'node is not on PATH, so that was not run.'
        return
    }

    Say ''

    # npm_config_yes is `npx --yes` spelled as the environment: on a cold cache
    # npx asks its own question -- "Ok to proceed?" -- and this one was
    # answered above, once. It is restored rather than left set, because under
    # `iex` the environment of this process is the caller's own.
    $savedYes = $env:npm_config_yes
    $code = 0
    try {
        $env:npm_config_yes = '1'
        $global:LASTEXITCODE = 0
        # Through Say for the reason Say exists: under `iex` output written to
        # the pipeline lands in whatever the caller was assembling. 2>&1 so
        # npx's diagnosis arrives as text rather than as error records a
        # caller's $ErrorActionPreference could turn into an exception.
        & npx skills add $Repo 2>&1 | ForEach-Object { Say "$_" }
        $code = $LASTEXITCODE
    } catch {
        Say "  $($_.Exception.Message)"
        $code = 9
    } finally {
        $env:npm_config_yes = $savedYes
    }

    if ($code -eq 0) {
        Say ''
        Say 'the skills are installed'
    } else {
        Say ''
        Say "npx skills add $Repo exited $code, so the skills are not installed."
        Say 'ank is, and it is exactly the ank this script installs when nobody is'
        Say 'asked anything at all.'
        Say ''
        Say 'Run that line again when you want them:'
        Say "  npx skills add $Repo"
    }
}

# ---------------------------------------------------------------------------
# Adopting ank where there is already code
# ---------------------------------------------------------------------------

# ADR-5fbd99bf6fd5's second offer, and the last question this script asks.
# Installing ank is the easy half. The half nobody had written down is what to
# say to an agent so that a repository with two years of history acquires a
# corpus worth having, and the moment after an install is the one moment the
# person is certainly reading.
#
# Three prompts, because the adoption has three moments: state as ADRs what the
# code already decided, so the constraints that exist implicitly become
# readable; turn a list of intentions into tasks carrying a scope and a
# criterion; and check what came out. The first one is the one the reader judges
# the tool on, which is why it is first.
#
# The same three prose blocks live in install.sh and in docs/getting-started.md,
# and a test holds the three copies character for character
# (crates/ank-cli/tests/adopt.rs). Prose duplicated in three files diverges, and
# this is the prose where divergence is worst: an installer teaching a prompt the
# documentation has since corrected. The markers below are what the test reads;
# the block between them is the one to edit, and the other two follow.
#
# A single-quoted here-string, so nothing in it is expanded: the text carries $
# and backticks in no place today, and a literal block is what keeps that from
# becoming a rule somebody has to remember. Its terminator sits at column zero
# because Windows PowerShell 5.1 requires it there.
# adopt-prompts:begin
$AdoptWalkthrough = @'
In a repository that already has code and no .ank, start with:

  ank init

Then paste these three into your agent, one at a time, and read what each
one produces before you send the next.

1. What the code already decided:

    Read this repository and write, as ank ADRs, the decisions its code
    has already made: the ones a newcomer would break without knowing
    they existed. One ADR per decision, each with a scope glob covering
    the files it binds and a constraint stated as a rule. Leave them
    proposed; I ratify them myself.

2. What is still owed:

    Read the TODOs, the open issues and the README of this repository,
    and turn what they promise into ank tasks. Give each one a scope
    glob and a done_criteria a test could settle, and use blocked_by
    only where a task genuinely waits on another.

3. What you now have:

    Run ank check and ank review here, then read every ADR back against
    the code its scope matches. Tell me which constraints the code
    already breaks and which scopes match no file, and change nothing
    until I have read your answer.

The same three are in docs/getting-started.md, which says what to expect
from each:

  https://github.com/haksolot/ank/blob/main/docs/getting-started.md
'@
# adopt-prompts:end

# The second question, asked on the same terms as the first: only with a human
# at a console, through [Console]::ReadLine so that a caller's `iex` pipeline is
# never what answers it, and with a default Enter accepts. Declining prints
# nothing -- not a shortened version, not a pointer to one. An offer that
# answers a no with half of a yes is an offer that was not really asked.
#
# Nothing in here is allowed to reach the caller as a failure. The call site
# wraps it, and the exit code is stamped after it returns.
function Invoke-AdoptionOffer {
    if (-not (Test-HumanAtTerminal)) { return }

    Say ''
    Say -NoNewline 'Print the three prompts that adopt ank in a repository you already have? [Y/n] '
    $answer = $null
    try { $answer = [Console]::ReadLine() } catch { $answer = $null }
    if ($null -eq $answer) {
        Say ''
        return
    }

    $reply = $answer.Trim()
    if ($reply -ne '' -and $reply -notmatch '^(y|yes)$') { return }

    Say ''
    # Split on either ending rather than on [Environment]::NewLine: this file is
    # fetched by `irm` on one machine and checked out by git on another, and the
    # bytes between two lines are not the same in both.
    foreach ($line in ($AdoptWalkthrough -split "\r?\n")) { Say $line }
    Say ''
}

# ---------------------------------------------------------------------------
# Platform
# ---------------------------------------------------------------------------

# Read out of the environment rather than off RuntimeInformation, and that is
# the deliberate half: an environment variable is what lets the refusal below be
# exercised on a machine that is not the platform being refused, so the test
# runs the real code path instead of a flag that bypasses it.
#
# PROCESSOR_ARCHITEW6432 is read first because a 32-bit PowerShell on 64-bit
# Windows reports x86 in the other variable, and installing nothing on a machine
# that is x64 would be a refusal about the process rather than about the
# hardware.
function Get-Architecture {
    if ($env:PROCESSOR_ARCHITEW6432) { return $env:PROCESSOR_ARCHITEW6432.ToUpperInvariant() }
    if ($env:PROCESSOR_ARCHITECTURE) { return $env:PROCESSOR_ARCHITECTURE.ToUpperInvariant() }
    return 'UNKNOWN'
}

# ARM64 takes the x64 archive on purpose. No native arm64 build is published,
# and Windows on ARM runs x64 under emulation transparently -- so refusing there
# would leave a working machine with no install to protect it from nothing. What
# is refused is the architecture that genuinely cannot run this binary: a 32-bit
# x86 Windows, where the emulation runs the other way and does not exist.
function Resolve-Target {
    $arch = Get-Architecture
    switch ($arch) {
        'AMD64' { return @{ Target = $WindowsTarget; Emulated = $false } }
        'ARM64' { return @{ Target = $WindowsTarget; Emulated = $true } }
        default {
            Fail 2 @(
                "no release archive for Windows/$arch.",
                '',
                "PROCESSOR_ARCHITECTURE reported: $arch",
                '',
                'The release carries one Windows build:',
                "  Windows x64         $WindowsTarget",
                '',
                'A 64-bit Windows runs it natively and Windows on ARM runs it under',
                'emulation; a 32-bit Windows runs neither.',
                '',
                'Everything the release carries is listed at:',
                "  $ReleasesUrl",
                '',
                'Nothing was installed.'
            )
        }
    }
}

# ---------------------------------------------------------------------------
# Hashing and unpacking
# ---------------------------------------------------------------------------

# Get-FileHash and Expand-Archive would both read better here, and neither is
# used, for a reason worth recording: they live in modules PowerShell autoloads,
# and autoloading is the part of this that can be broken by the environment
# rather than by the machine. Measured while writing this file -- a 5.1 started
# from a session whose PSModulePath pointed at PowerShell 7's module directories
# resolved neither cmdlet, and the install died after downloading the archive,
# on the step that exists to verify it.
#
# The .NET types below are in the runtime itself. Nothing is searched for, so
# there is nothing to shadow, and the two operations this script must never
# skip are the two that stop depending on a lookup.

function Get-Sha256 {
    param([string]$Path)

    $sha = [System.Security.Cryptography.SHA256]::Create()
    $stream = [System.IO.File]::OpenRead($Path)
    try {
        $bytes = $sha.ComputeHash($stream)
    } finally {
        $stream.Dispose()
        $sha.Dispose()
    }
    return ([System.BitConverter]::ToString($bytes) -replace '-', '').ToLowerInvariant()
}

function Expand-Zip {
    param([string]$Path, [string]$Destination)

    # Built into PowerShell 7; on 5.1 the assembly is present and simply has to
    # be asked for. Loading one already loaded is a no-op, so this is not
    # guarded by a version test that would be one more thing to be wrong.
    Add-Type -AssemblyName System.IO.Compression.FileSystem -ErrorAction SilentlyContinue
    [System.IO.Compression.ZipFile]::ExtractToDirectory($Path, $Destination)
}

# Windows locks a running executable, so an upgrade run while another shell has
# one of these open cannot write over the file -- but it can rename it, which is
# the one thing Windows does allow. The old binary is moved aside and the new
# one takes the name; the stale copies are swept on the next run, since the
# process holding one is still holding it now.
#
# A function rather than an inlined block: what it does is subtle enough to be
# worth reading on its own, and its one caller is easier to read for not
# carrying it. Returns the message of the failure it could not work around, or
# $null.
function Move-IntoPlace {
    param([string]$Source, [string]$Destination, [string]$Name)

    # Out-Null and not for tidiness: what this function returns is read as a
    # message, so a pipeline above it that emitted anything would turn that
    # message into an array and the caller's `if` into a lie.
    Get-ChildItem -Path (Split-Path -Parent $Destination) -Filter "$Name.old-*" `
        -ErrorAction SilentlyContinue |
        ForEach-Object { Remove-Item -Path $_.FullName -Force -ErrorAction SilentlyContinue } |
        Out-Null

    try {
        Move-Item -Path $Source -Destination $Destination -Force -ErrorAction Stop
        return $null
    } catch {
        $aside = "$Destination.old-" + [System.IO.Path]::GetRandomFileName()
        try {
            Move-Item -Path $Destination -Destination $aside -Force -ErrorAction Stop
            Move-Item -Path $Source -Destination $Destination -Force -ErrorAction Stop
            Say "the running $Name was moved aside; it is swept on the next install"
            return $null
        } catch {
            return "$($_.Exception.Message)"
        }
    }
}

# ---------------------------------------------------------------------------
# Downloading
# ---------------------------------------------------------------------------

# Returns the HTTP status when the request produced one, so that "this release
# does not carry that archive" and "the network is down" stay different
# messages: the 404 is what tells them apart.
function Get-File {
    param([string]$Url, [string]$OutFile)

    try {
        Invoke-WebRequest -Uri $Url -OutFile $OutFile -UseBasicParsing -ErrorAction Stop
        return @{ Ok = $true; Status = '' }
    } catch [System.Net.WebException] {
        $status = ''
        if ($_.Exception.Response) { $status = [int]$_.Exception.Response.StatusCode }
        return @{ Ok = $false; Status = "$status" }
    } catch {
        # PowerShell 7 wraps the failure in HttpResponseException instead, and
        # keeps the status on the same property.
        $status = ''
        if (($_.Exception | Get-Member -Name Response) -and $_.Exception.Response) {
            $status = [int]$_.Exception.Response.StatusCode
        }
        return @{ Ok = $false; Status = "$status" }
    }
}

# The tag of the latest release, read off the redirect rather than through the
# API: /releases/latest redirects to /releases/tag/<tag>, which costs no rate
# limit, where api.github.com allows sixty unauthenticated calls an hour per
# address and an office NAT can be out of them before anyone types this.
#
# HttpWebRequest and not Invoke-WebRequest, because -MaximumRedirection 0 is one
# of the places 5.1 and 7 disagree: one returns the response and the other
# throws, and reading the Location header off the raw request behaves the same
# on both.
function Resolve-LatestTag {
    $tag = ''
    try {
        $request = [System.Net.HttpWebRequest]::Create("$ReleasesUrl/latest")
        $request.AllowAutoRedirect = $false
        $request.Method = 'HEAD'
        $request.UserAgent = 'ank-install.ps1'
        $response = $request.GetResponse()
        $location = $response.Headers['Location']
        $response.Close()
        if ($location) { $tag = $location.Split('/')[-1] }
    } catch {
        $tag = ''
    }

    if ($tag -notmatch '^v') {
        Fail 3 @(
            'could not work out which release is the latest.',
            '',
            'Name one instead:',
            "  & ([scriptblock]::Create((irm $RawUrl))) -Version v0.2.0",
            '',
            'The releases are listed at:',
            "  $ReleasesUrl"
        )
    }
    return $tag
}

# ---------------------------------------------------------------------------
# Arguments
# ---------------------------------------------------------------------------

if ($Help) {
    Show-Usage
    return
}

# 5.1 is what a clean Windows 10 or 11 ships and what CI runs this against, so
# it is the floor this file claims. Older is refused rather than attempted: the
# .NET types above reach back further, but nothing here has ever been run there
# and an install that half works is worse than one that says no. Checked before
# anything is downloaded, so a machine that cannot finish is told before it
# spends the bytes.
if ($PSVersionTable.PSVersion.Major -lt 5) {
    Fail 5 @(
        "this script needs PowerShell 5.1 or newer, and this is $($PSVersionTable.PSVersion).",
        '',
        'Windows 10 and 11 ship 5.1. On an older Windows, install either:',
        '  Windows Management Framework 5.1, or',
        '  PowerShell 7:  https://aka.ms/powershell',
        '',
        'Nothing was installed.'
    )
}

if (-not $Version -and $env:ANK_VERSION) { $Version = $env:ANK_VERSION }
if (-not $Dir -and $env:ANK_INSTALL_DIR) { $Dir = $env:ANK_INSTALL_DIR }
$BaseUrl = $env:ANK_BASE_URL

if ($BaseUrl) {
    if ($BaseUrl -notmatch '^https?://') {
        Fail 1 @(
            "ANK_BASE_URL must start with http:// or https://, got: $BaseUrl",
            '',
            'Unset it to fetch from the GitHub release:',
            '  $env:ANK_BASE_URL = $null'
        )
    }
    if (-not $Version) {
        Fail 1 @(
            'ANK_BASE_URL is set, so the version has to be named.',
            '',
            'Only GitHub can be asked which release is the latest; a mirror cannot.',
            "  & ([scriptblock]::Create((irm $RawUrl))) -Version v0.2.0"
        )
    }
}

# ---------------------------------------------------------------------------
# Welcome, then install
# ---------------------------------------------------------------------------

# Before anything is asked of the network, which is what "before the download
# starts" has to mean here: the first request this script makes is the HEAD
# that resolves the latest tag, and it is below.
#
# Wrapped, because a host that cannot place a cursor is not a reason to fail an
# install: nothing has been downloaded yet and nothing below depends on a frame
# having been drawn.
if (Test-HumanAtTerminal) {
    try { Show-Logo } catch { }
}

# Both of these are session state, and under `iex` the session belongs to the
# caller: they are restored in the finally below rather than left changed by an
# install. TLS 1.2 because 5.1 defaults to protocols GitHub no longer accepts,
# and the progress bar because on 5.1 it costs more time than the download.
$savedProtocol = [Net.ServicePointManager]::SecurityProtocol
$savedProgress = $ProgressPreference
$tmp = ''

try {
    [Net.ServicePointManager]::SecurityProtocol = `
        [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
    $ProgressPreference = 'SilentlyContinue'

    $resolved = Resolve-Target
    $target = $resolved.Target

    if ($Version) {
        # "v0.2.0" and "0.2.0" are the same request. The tag carries the v and
        # the archive name does not, and making the caller know which is which
        # is the kind of detail a released script gets to absorb.
        $tag = 'v' + ($Version -replace '^v', '')
    } else {
        $tag = Resolve-LatestTag
    }
    $bare = $tag -replace '^v', ''

    $archive = "ank-$bare-$target.zip"
    $root = if ($BaseUrl) { $BaseUrl } else { $DefaultBaseUrl }
    $url = "$root/$tag/$archive"

    Say "ank $tag  $target"
    if ($resolved.Emulated) {
        Say 'no native arm64 build is published; Windows runs this one under emulation'
    }

    $tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("ank-install-" + [System.IO.Path]::GetRandomFileName())
    New-Item -ItemType Directory -Path $tmp -Force | Out-Null

    # The checksum first: it is the smaller of the two files, so a release that
    # does not carry this target says so before megabytes move. It is also the
    # request that answers "is this platform in this release", which is a
    # question the caller deserves answered rather than left as a stalled
    # download.
    $shaPath = Join-Path $tmp 'sha256'
    $got = Get-File -Url "$url.sha256" -OutFile $shaPath
    if (-not $got.Ok) {
        if ($got.Status -eq '404') {
            Fail 3 @(
                "$tag does not carry an archive for this platform.",
                '',
                "  looked for:  $archive",
                "  at:          $url",
                '',
                'The platform is one this project builds for, so this is about the',
                "release and not about the machine. What $tag carries is listed at:",
                "  $ReleasesUrl/tag/$tag",
                '',
                'Pick a release that carries it:',
                "  & ([scriptblock]::Create((irm $RawUrl))) -Version <version>",
                '',
                'Nothing was installed.'
            )
        }
        $suffix = if ($got.Status) { " (HTTP $($got.Status))" } else { '' }
        Fail 3 @(
            "could not download the checksum for $archive$suffix.",
            '',
            "  $url.sha256",
            '',
            'Nothing was installed.'
        )
    }

    # sha256sum's own format, which is what release.yml publishes: the hash,
    # whitespace, the file name.
    $firstLine = (Get-Content -Path $shaPath -TotalCount 1)
    $expected = ''
    if ($firstLine) { $expected = ($firstLine.Trim() -split '\s+')[0].ToLowerInvariant() }

    # A captive portal answering every request with a login page is a real way
    # to get a .sha256 that parses into something. Sixty-four hex characters, or
    # this was not the release answering.
    if ($expected -notmatch '^[0-9a-f]{64}$') {
        $readBack = if ($expected) { $expected } else { '<empty>' }
        Fail 4 @(
            "the published checksum for $archive is not a SHA-256.",
            '',
            "  $url.sha256",
            "  read back:  $readBack",
            '',
            'Something other than the release answered that request. Nothing was',
            'unpacked and nothing was installed.'
        )
    }

    $archivePath = Join-Path $tmp $archive
    $got = Get-File -Url $url -OutFile $archivePath
    if (-not $got.Ok) {
        $suffix = if ($got.Status) { " (HTTP $($got.Status))" } else { '' }
        Fail 3 @(
            "could not download $archive$suffix.",
            '',
            "  $url",
            '',
            'Nothing was installed.'
        )
    }

    $actual = Get-Sha256 -Path $archivePath

    # Before unpacking, and this is what the file is for. A script that
    # downloads an executable over the network and runs it without checking the
    # hash published beside it is a supply chain with a hole in the middle.
    #
    # What the check buys, stated honestly: it catches a truncated or corrupted
    # download, a mirror serving the wrong file, and an archive that is not the
    # one the release recorded. It is not a signature -- the hash comes from the
    # same host as the archive -- which is why the default host is GitHub over
    # TLS and why ANK_BASE_URL is documented as a mirror rather than as a
    # default.
    if ($expected -ne $actual) {
        Fail 4 @(
            "checksum mismatch, refusing to unpack $archive.",
            '',
            "  expected:   $expected",
            "  actual:     $actual",
            "  published:  $url.sha256",
            '',
            'The download does not match the hash the release published beside it.',
            'Nothing was unpacked and nothing was installed.',
            '',
            'Retry once, in case the transfer was truncated. If it happens again, do',
            'not install this archive, and say so:',
            "  https://github.com/$Repo/security"
        )
    }

    Say "checksum ok  $actual"

    try {
        Expand-Zip -Path $archivePath -Destination $tmp
    } catch {
        Fail 3 @(
            "could not unpack $archive.",
            '',
            "  $($_.Exception.Message)",
            '',
            'Nothing was installed.'
        )
    }

    # The layout release.yml packages: one directory named after the archive,
    # carrying the executable beside README.md, LICENSE and SKILL.md.
    #
    # ank.exe is required and the rest of the directory is not read at all. An
    # archive published before ADR-1ea31c2f3c5a carries a second executable
    # there, and the answer to it is the same as the answer to README.md: it is
    # not what was asked for, so it is not moved anywhere. A check that refused
    # an archive holding more than this would refuse every release published so
    # far.
    $unpacked = Join-Path $tmp "ank-$bare-$target"
    $binary = Join-Path $unpacked 'ank.exe'
    if (-not (Test-Path -Path $binary -PathType Leaf)) {
        Fail 3 @(
            "$archive does not contain ank.exe where this script expected it.",
            '',
            "  looked for:  ank-$bare-$target\ank.exe",
            '',
            'Nothing was installed.'
        )
    }

    if (-not $Dir) {
        if (-not $env:LOCALAPPDATA) {
            Fail 1 @(
                'LOCALAPPDATA is not set, so there is no default install directory.',
                '',
                'Name one:',
                "  & ([scriptblock]::Create((irm $RawUrl))) -Dir C:\tools\ank"
            )
        }
        $Dir = Join-Path $env:LOCALAPPDATA 'Programs\ank'
    }

    try {
        New-Item -ItemType Directory -Path $Dir -Force -ErrorAction Stop | Out-Null
    } catch {
        Fail 1 @(
            "could not create $Dir.",
            '',
            "  $($_.Exception.Message)",
            '',
            'Install somewhere writable, or run this from a shell that can write there:',
            "  & ([scriptblock]::Create((irm $RawUrl))) -Dir `$env:LOCALAPPDATA\Programs\ank"
        )
    }

    $destination = Join-Path $Dir 'ank.exe'

    $failure = Move-IntoPlace -Source $binary -Destination $destination -Name 'ank.exe'
    if ($failure) {
        Fail 1 @(
            "could not write $destination.",
            '',
            "  $failure",
            '',
            'Something is holding the file open and it could not be renamed either.',
            'Close any shell running ank and try again.',
            '',
            'Nothing was installed.'
        )
    }

    $installedVersion = ''
    try {
        $installedVersion = (& $destination --version 2>$null | Select-Object -First 1)
    } catch {
        $installedVersion = ''
    }

    Say ''
    Say "installed  $destination"
    if ($installedVersion) { Say "           $installedVersion" }

    # The last way left to leave a caller without a working `ank`: a binary in a
    # directory nothing looks in. Naming the command to run is the difference
    # between an install that worked and an install that appears not to have
    # run. The user PATH is not written by this script: it is a persistent
    # change to the caller's account, and an installer that makes one without
    # being asked is one they cannot undo by deleting what it installed.
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $onPath = ($env:Path -split ';' | Where-Object { $_.TrimEnd('\') -ieq $Dir.TrimEnd('\') })

    if ($onPath) {
        Say ''
        Say "$Dir is on your PATH. Run: ank help"
    } else {
        Say ''
        Say "$Dir is not on your PATH, so ``ank`` is not a command yet."
        Say 'Add it for your account by running this once:'
        Say ''
        Say "  [Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path','User') + ';$Dir', 'User')"
        Say ''
        Say 'then open a new terminal. In this one:'
        Say ''
        Say "  `$env:Path += ';$Dir'"
        if ($userPath -and $userPath.Length -gt 1800) {
            Say ''
            Say 'Your user PATH is close to the length Windows truncates at, so check it'
            Say 'after adding: a truncated PATH loses entries other tools put there.'
        }
    }

    # Wrapped, and it is the whole guarantee in two lines rather than one.
    #
    # The try is the half install.sh spells `offer_skills || :`: a question
    # asked after a successful install may not turn that install into a
    # failure, so nothing thrown in there gets out of here.
    #
    # The stamp is the half that has no counterpart in install.sh, and it is
    # the one that would have been missed by reading. `pwsh -File install.ps1`
    # exits with $LASTEXITCODE, which every native command run above sets: with
    # nothing here, an npx that failed would silently become this script's exit
    # code and the caller would read a green install as red. Assigned in the
    # global scope, because an assignment inside a script writes a script-local
    # variable that shadows it and changes nothing the host reads.
    try { Invoke-SkillOffer } catch { }
    try { Invoke-AdoptionOffer } catch { }
    $global:LASTEXITCODE = 0
} finally {
    if ($tmp -and (Test-Path -Path $tmp)) {
        Remove-Item -Path $tmp -Recurse -Force -ErrorAction SilentlyContinue
    }
    $ProgressPreference = $savedProgress
    [Net.ServicePointManager]::SecurityProtocol = $savedProtocol
}
