//! The three prompts that adopt ank in a repository that already has code.
//!
//! They exist in three places -- `install.sh`, `install.ps1` and
//! `docs/getting-started.md` -- and that is the whole reason this file is here.
//! Prose duplicated in three files diverges, and this is the prose where
//! divergence is worst: an installer teaching a prompt the documentation has
//! since corrected, on the one route where the reader has no way of knowing the
//! two disagree. So one of the three is the source and the other two are copies,
//! and the equality is asserted rather than remembered.
//!
//! Which one is the source is an implementation choice; that there is exactly
//! one is not. Here it is the block between the markers in `install.sh`: the
//! other two follow it, and this test is what says so out loud.
//!
//! The rest is what the criterion of TASK-567084d21d2b asks of the two
//! installers around that block -- a ceiling of forty lines, one question, the
//! terminal predicate ahead of it, and nothing printed to somebody who declined.

use std::path::PathBuf;
use std::process::Command;
use std::{fs, process::Stdio};

fn repo_file(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    let text = fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    // Neither installer is pinned to LF by `.gitattributes`, so a checkout on
    // Windows hands this test CRLF. What the criterion is about is the
    // characters, and a line ending git chose is not one of them.
    text.replace("\r\n", "\n")
}

// ---------------------------------------------------------------------------
// The markers, and what is read between them
// ---------------------------------------------------------------------------

/// The lines strictly between the two marker lines, which every one of the three
/// files carries in the comment syntax it happens to have.
///
/// The marker is matched as a substring rather than as a whole line, because
/// markdown spells a comment `<!-- ... -->` and neither script does.
fn marked_region(file: &str, text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.split('\n').collect();
    let begin = lines
        .iter()
        .position(|l| l.contains("adopt-prompts:begin"))
        .unwrap_or_else(|| panic!("{file}: no adopt-prompts:begin marker"));
    let end = lines
        .iter()
        .position(|l| l.contains("adopt-prompts:end"))
        .unwrap_or_else(|| panic!("{file}: no adopt-prompts:end marker"));
    assert!(begin < end, "{file}: the markers are in the wrong order");
    lines[begin + 1..end]
        .iter()
        .map(|l| l.to_string())
        .collect()
}

/// A prompt is a maximal run of lines opening on four spaces.
///
/// Four is what a markdown indented code block costs, so the document can carry
/// the prompts as blocks a reader copies while the two installers carry the same
/// bytes as the indentation that sets a pasteable block apart from the prose
/// around it. Everything else inside the region -- the heredoc that wraps it in
/// `install.sh`, the here-string in `install.ps1`, the paragraph between two
/// prompts in the document -- opens on something else and is skipped.
fn prompts(file: &str, region: &[String]) -> Vec<String> {
    let mut found = Vec::new();
    let mut current: Vec<String> = Vec::new();
    for line in region {
        if line.starts_with("    ") && !line.trim().is_empty() {
            current.push(line.clone());
        } else if !current.is_empty() {
            found.push(current.join("\n"));
            current.clear();
        }
    }
    if !current.is_empty() {
        found.push(current.join("\n"));
    }
    assert_eq!(
        found.len(),
        3,
        "{file}: expected three prompts between the markers, found {}",
        found.len()
    );
    found
}

fn sh_prompts() -> Vec<String> {
    let text = repo_file("install.sh");
    prompts("install.sh", &marked_region("install.sh", &text))
}

fn ps_prompts() -> Vec<String> {
    let text = repo_file("install.ps1");
    prompts("install.ps1", &marked_region("install.ps1", &text))
}

fn doc_prompts() -> Vec<String> {
    let text = repo_file("docs/getting-started.md");
    prompts(
        "docs/getting-started.md",
        &marked_region("docs/getting-started.md", &text),
    )
}

// ---------------------------------------------------------------------------
// The drift the criterion names
// ---------------------------------------------------------------------------

/// The three copies are equal character for character.
///
/// Asserted prompt by prompt and not as one blob, so a failure names which of
/// the three moved rather than handing over two walls of text to diff by eye.
#[test]
fn the_three_copies_of_each_prompt_are_identical() {
    let sh = sh_prompts();
    let ps = ps_prompts();
    let doc = doc_prompts();

    for (i, source) in sh.iter().enumerate() {
        assert_eq!(
            source,
            &ps[i],
            "prompt {} differs between install.sh and install.ps1",
            i + 1
        );
        assert_eq!(
            source,
            &doc[i],
            "prompt {} differs between install.sh and docs/getting-started.md",
            i + 1
        );
    }
}

/// A prompt that says nothing is a prompt that passes the test above.
///
/// The equality is the criterion; this is the floor under it. Three blocks of
/// four spaces would be identical in all three files and would adopt nothing.
#[test]
fn each_prompt_is_an_instruction_and_not_a_placeholder() {
    for (i, prompt) in sh_prompts().iter().enumerate() {
        let words = prompt.split_whitespace().count();
        assert!(
            words >= 30,
            "prompt {} is {words} words, which is not an instruction",
            i + 1
        );
        assert!(
            prompt.contains("ank"),
            "prompt {} never names ank, so it adopts nothing",
            i + 1
        );
    }
}

/// The walkthrough is the literal block, and no line of it is assembled.
///
/// Read out of the heredoc in `install.sh` and out of the here-string in
/// `install.ps1`, both of them quoted so that neither shell expands anything
/// inside. The two blocks are compared whole and not only through their prompts:
/// the framing carries the first command to run, and an installer whose framing
/// drifted would send a Windows reader somewhere else.
fn literal_block(file: &str, open_suffix: &str, close: &str) -> Vec<String> {
    let text = repo_file(file);
    let region = marked_region(file, &text);
    let open = region
        .iter()
        .position(|l| l.trim_end().ends_with(open_suffix))
        .unwrap_or_else(|| panic!("{file}: no line ending in {open_suffix} between the markers"));
    let end = region
        .iter()
        .position(|l| l.trim_end() == close)
        .unwrap_or_else(|| panic!("{file}: no line {close} between the markers"));
    assert!(
        open < end,
        "{file}: the literal block opens after it closes"
    );
    region[open + 1..end].to_vec()
}

fn sh_walkthrough() -> Vec<String> {
    literal_block("install.sh", "<<'ADOPT_EOF'", "ADOPT_EOF")
}

fn ps_walkthrough() -> Vec<String> {
    literal_block("install.ps1", "@'", "'@")
}

#[test]
fn both_installers_print_the_same_walkthrough() {
    assert_eq!(
        sh_walkthrough(),
        ps_walkthrough(),
        "install.sh and install.ps1 print different walkthroughs"
    );
}

/// At most forty lines, which is the criterion's ceiling and the reason there is
/// one: what does not fit on a screen after an install is not read.
///
/// Two more than the block itself, because both offers put a blank line on each
/// side of it before it reaches the terminal.
#[test]
fn the_walkthrough_fits_in_forty_lines() {
    let printed = sh_walkthrough().len() + 2;
    assert!(
        printed <= 40,
        "the walkthrough prints {printed} lines, over the ceiling of forty"
    );
}

/// The first command a reader runs is in there, since the criterion asks the
/// walkthrough for it and three prompts on their own would leave a repository
/// with no corpus to write into.
#[test]
fn the_walkthrough_carries_the_first_command() {
    let block = sh_walkthrough().join("\n");
    assert!(
        block.contains("\n  ank init\n"),
        "the walkthrough never names `ank init` as a command to run"
    );
}

// ---------------------------------------------------------------------------
// Asked once, only to a human, and silent to a no
// ---------------------------------------------------------------------------

/// The body of a shell function, from its opening line to the `}` at column
/// zero. Enough for this file, and it is what makes the assertions below about
/// the offer rather than about the script.
fn sh_function(name: &str) -> String {
    let text = repo_file("install.sh");
    let open = format!("\n{name}() {{\n");
    let start = text
        .find(&open)
        .unwrap_or_else(|| panic!("install.sh: no function {name}"))
        + open.len();
    let rest = &text[start..];
    let end = rest
        .find("\n}\n")
        .unwrap_or_else(|| panic!("install.sh: {name} is never closed"));
    rest[..end].to_string()
}

/// The body of a PowerShell function, on the same terms: to the `}` at column
/// zero.
fn ps_function(name: &str) -> String {
    let text = repo_file("install.ps1");
    let open = format!("\nfunction {name} {{\n");
    let start = text
        .find(&open)
        .unwrap_or_else(|| panic!("install.ps1: no function {name}"))
        + open.len();
    let rest = &text[start..];
    let end = rest
        .find("\n}\n")
        .unwrap_or_else(|| panic!("install.ps1: {name} is never closed"));
    rest[..end].to_string()
}

/// The terminal predicate comes first, and it is the same one the logo and the
/// skills offer read.
///
/// One predicate is what makes `--no-welcome` and an interactive run that
/// declined everything leave the same machine behind (ADR-5fbd99bf6fd5): a
/// second gate is a second thing able to disagree with this one.
#[test]
fn the_offer_is_gated_on_a_terminal_before_anything_else() {
    let sh = sh_function("offer_adoption");
    let first = sh.lines().next().unwrap().trim();
    assert_eq!(
        first, "human_at_terminal || return 0",
        "install.sh: offer_adoption does something before testing for a terminal"
    );

    let ps = ps_function("Invoke-AdoptionOffer");
    let first = ps.lines().next().unwrap().trim();
    assert_eq!(
        first, "if (-not (Test-HumanAtTerminal)) { return }",
        "install.ps1: Invoke-AdoptionOffer does something before testing for a console"
    );
}

/// The question is read from the controlling terminal and from nowhere else.
///
/// This is the trap ADR-5fbd99bf6fd5 exists to name. Under `curl ... | sh`
/// standard input is the script: a bare `read` would swallow the rest of the
/// file and execute none of it, and it would do so only on the route people
/// actually use. Windows spells the same rule `[Console]::ReadLine`, which reads
/// the console rather than the pipeline a caller's `iex` is running.
#[test]
fn the_question_is_read_from_the_terminal() {
    let sh = sh_function("offer_adoption");
    assert!(
        sh.contains("read -r adopt_answer < /dev/tty"),
        "install.sh: the adoption question does not read from /dev/tty"
    );

    let ps = ps_function("Invoke-AdoptionOffer");
    assert!(
        ps.contains("[Console]::ReadLine()"),
        "install.ps1: the adoption question does not read the console"
    );
}

/// Asked once. Not once per platform, once per run: a second copy of the
/// question in either file is a second time the reader is asked.
#[test]
fn the_question_is_asked_once() {
    let question =
        "Print the three prompts that adopt ank in a repository you already have? [Y/n] ";
    assert_eq!(
        repo_file("install.sh").matches(question).count(),
        1,
        "install.sh does not ask the adoption question exactly once"
    );
    assert_eq!(
        repo_file("install.ps1").matches(question).count(),
        1,
        "install.ps1 does not ask the adoption question exactly once"
    );

    assert_eq!(
        repo_file("install.sh")
            .matches("\noffer_adoption || :\n")
            .count(),
        1,
        "install.sh does not run the adoption offer exactly once"
    );
    assert_eq!(
        repo_file("install.ps1")
            .matches("try { Invoke-AdoptionOffer } catch { }")
            .count(),
        1,
        "install.ps1 does not run the adoption offer exactly once"
    );
}

/// Enter accepts, and everything unrecognised declines.
///
/// ADR-5fbd99bf6fd5 asks every question for a default Enter accepts. The empty
/// answer is that default in both files.
#[test]
fn enter_accepts_the_offer() {
    assert!(
        sh_function("offer_adoption").contains(r#""" | y | Y | yes | Yes | YES"#),
        "install.sh: Enter is not the default answer to the adoption question"
    );
    assert!(
        ps_function("Invoke-AdoptionOffer")
            .contains("if ($reply -ne '' -and $reply -notmatch '^(y|yes)$') { return }"),
        "install.ps1: Enter is not the default answer to the adoption question"
    );
}

/// Declining prints nothing.
///
/// Not a shortened version, not a pointer to one: the decline branch returns,
/// and every line the offer writes is written after it. Asserted by position,
/// which is what a reader of either file checks by eye and what neither language
/// lets a type say.
#[test]
fn declining_prints_nothing() {
    let sh = sh_function("offer_adoption");
    let decline = sh
        .find("*) return 0 ;;")
        .expect("install.sh: offer_adoption has no decline branch");
    let call = sh
        .find("adopt_walkthrough")
        .expect("install.sh: offer_adoption never prints the walkthrough");
    assert!(
        decline < call,
        "install.sh: the walkthrough is reachable before the decline returns"
    );
    assert!(
        !sh[..decline].contains("say \"")
            || sh[..decline].matches("say \"\"").count() == sh[..decline].matches("say \"").count(),
        "install.sh: a line with content is printed before the reader has answered"
    );

    let ps = ps_function("Invoke-AdoptionOffer");
    let decline = ps
        .find("-notmatch '^(y|yes)$') { return }")
        .expect("install.ps1: Invoke-AdoptionOffer has no decline branch");
    let call = ps
        .find("foreach ($line in ($AdoptWalkthrough")
        .expect("install.ps1: Invoke-AdoptionOffer never prints the walkthrough");
    assert!(
        decline < call,
        "install.ps1: the walkthrough is reachable before the decline returns"
    );
}

/// Having no terminal prints nothing, and it is the same sentence as the one
/// above: the predicate returns before the question is asked, so a runner, a
/// Dockerfile or a provisioning script sees the installer it saw before this
/// offer existed. Covered by the gating test; asserted here on the flag, which
/// is the half a reader can turn on deliberately.
#[test]
fn no_welcome_reaches_the_adoption_offer() {
    assert!(
        sh_function("human_at_terminal").contains(r#"[ "$no_welcome" = no ] || return 1"#),
        "install.sh: --no-welcome no longer closes the predicate the offer reads"
    );
    assert!(
        ps_function("Test-HumanAtTerminal").contains("if ($NoWelcome) { return $false }"),
        "install.ps1: -NoWelcome no longer closes the predicate the offer reads"
    );
}

// ---------------------------------------------------------------------------
// The scripts still parse
// ---------------------------------------------------------------------------

/// Editing an 844-line shell script by hand is how a shell script stops being
/// one, and the failure surfaces on a stranger's machine rather than here. `sh
/// -n` parses without running, which is exactly the reach this test wants.
///
/// Skipped where there is no `sh`, which on this matrix means Windows: the file
/// is parsed on the two platforms that run it.
#[test]
fn install_sh_parses() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let out = match Command::new("sh")
        .arg("-n")
        .arg(root.join("install.sh"))
        .stdin(Stdio::null())
        .output()
    {
        Ok(out) => out,
        Err(_) => return,
    };
    assert!(
        out.status.success(),
        "sh -n install.sh: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The same for `install.ps1`, through the parser rather than through a run: the
/// here-string added for the walkthrough has a terminator PowerShell wants at
/// column zero, and getting that wrong is a parse error on a machine no CI job
/// on Linux would ever reach.
///
/// Skipped where there is no PowerShell, which is every platform but the one
/// this file is for.
#[test]
fn install_ps1_parses() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let script = root.join("install.ps1");
    let program = ["pwsh", "powershell"].into_iter().find(|p| {
        Command::new(p)
            .arg("-Help")
            .stdin(Stdio::null())
            .output()
            .is_ok()
    });
    let Some(program) = program else {
        return;
    };
    let out = Command::new(program)
        .args(["-NoProfile", "-NonInteractive", "-Command"])
        .arg(format!(
            "$e = $null; [void][System.Management.Automation.Language.Parser]::ParseFile('{}', [ref]$null, [ref]$e); if ($e.Count) {{ $e | ForEach-Object {{ $_.Message }}; exit 1 }}",
            script.display()
        ))
        .stdin(Stdio::null())
        .output()
        .expect("running PowerShell");
    assert!(
        out.status.success(),
        "install.ps1 does not parse: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
