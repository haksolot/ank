<#
Install ank from a GitHub release.

  irm https://raw.githubusercontent.com/haksolot/ank/main/install.ps1 | iex

  & ([scriptblock]::Create((irm https://raw.githubusercontent.com/haksolot/ank/main/install.ps1))) -Version v0.2.0

The Windows counterpart of install.sh, and the same contract: fetch the archive
the release published, verify it against the .sha256 published beside it, unpack
it, and never end in silence. What differs is only what Windows spells
differently.

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
    param([string]$Line = '')
    Write-Host $Line
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
    Say '  -Help               print this and exit'
    Say ''
    Say 'environment:'
    Say '  ANK_VERSION         same as -Version'
    Say '  ANK_INSTALL_DIR     same as -Dir'
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
    Say 'exit codes:'
    Say '  1 usage   2 unsupported platform   3 download   4 checksum   5 missing runtime'
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
# Install
# ---------------------------------------------------------------------------

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
    # carrying the binary beside README.md, LICENSE and SKILL.md.
    $binary = Join-Path $tmp "ank-$bare-$target\ank.exe"
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

    # Windows locks a running executable, so an upgrade run while another shell
    # has ank open cannot write over the file -- but it can rename it, which is
    # the one thing Windows does allow. The old binary is moved aside and the
    # new one takes the name; the stale copies are swept on the next run, since
    # the process holding one is still holding it now.
    Get-ChildItem -Path $Dir -Filter 'ank.exe.old-*' -ErrorAction SilentlyContinue |
        ForEach-Object { Remove-Item -Path $_.FullName -Force -ErrorAction SilentlyContinue }

    try {
        Move-Item -Path $binary -Destination $destination -Force -ErrorAction Stop
    } catch {
        $aside = "$destination.old-" + [System.IO.Path]::GetRandomFileName()
        try {
            Move-Item -Path $destination -Destination $aside -Force -ErrorAction Stop
            Move-Item -Path $binary -Destination $destination -Force -ErrorAction Stop
            Say 'the running ank.exe was moved aside; it is swept on the next install'
        } catch {
            Fail 1 @(
                "could not write $destination.",
                '',
                "  $($_.Exception.Message)",
                '',
                'Something is holding the file open and it could not be renamed either.',
                'Close any shell running ank and try again.',
                '',
                'Nothing was installed.'
            )
        }
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
} finally {
    if ($tmp -and (Test-Path -Path $tmp)) {
        Remove-Item -Path $tmp -Recurse -Force -ErrorAction SilentlyContinue
    }
    $ProgressPreference = $savedProgress
    [Net.ServicePointManager]::SecurityProtocol = $savedProtocol
}
