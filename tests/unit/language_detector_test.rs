use code_analyzer::metrics::language::LanguageDetector;

#[test]
fn test_detect_by_filename() {
    let detector = LanguageDetector::new();
    
    assert_eq!(detector.detect_by_filename(".gitignore"), "GitConfig");
    assert_eq!(detector.detect_by_filename("Dockerfile"), "Docker");
    assert_eq!(detector.detect_by_filename("web.config"), "ASP.NET");
    assert_eq!(detector.detect_by_filename("global.asax"), "ASP.NET");
    assert_eq!(detector.detect_by_filename("AssemblyInfo.cs"), "C#");
    assert_eq!(detector.detect_by_filename("AssemblyInfo.vb"), "VisualBasic");
    assert_eq!(detector.detect_by_filename("NuGet.config"), "DotNetProject");
    assert_eq!(detector.detect_by_filename("unknown"), "Other");
}

#[test]
fn test_detect_language() {
    let detector = LanguageDetector::new();

    assert_eq!(detector.detect_language("rs"), "Rust");
    assert_eq!(detector.detect_language("js"), "JavaScript");
    assert_eq!(detector.detect_language("jsx"), "JavaScript");
    assert_eq!(detector.detect_language("ts"), "TypeScript");
    assert_eq!(detector.detect_language("tsx"), "TypeScript");
    assert_eq!(detector.detect_language("py"), "Python");
    assert_eq!(detector.detect_language("cs"), "C#");
    assert_eq!(detector.detect_language("vb"), "VisualBasic");
    assert_eq!(detector.detect_language("fs"), "FSharp");
    assert_eq!(detector.detect_language("xaml"), "XAML");
    assert_eq!(detector.detect_language("cshtml"), "Razor");
    assert_eq!(detector.detect_language("razor"), "Razor");
    assert_eq!(detector.detect_language("aspx"), "ASP.NET");
    assert_eq!(detector.detect_language("ascx"), "ASP.NET");
    assert_eq!(detector.detect_language("csproj"), "DotNetProject");
    assert_eq!(detector.detect_language("vbproj"), "DotNetProject");
    assert_eq!(detector.detect_language("fsproj"), "DotNetProject");
    assert_eq!(detector.detect_language("sln"), "DotNetProject");
    assert_eq!(detector.detect_language("unknown"), "Other");
}

#[test]
fn test_get_comment_syntax() {
    let detector = LanguageDetector::new();

    let (line, block_start, block_end) = detector.get_comment_syntax("Rust");
    assert_eq!(line, "//");
    assert_eq!(block_start, "/*");
    assert_eq!(block_end, "*/");

    let (line, block_start, block_end) = detector.get_comment_syntax("Python");
    assert_eq!(line, "#");
    assert_eq!(block_start, "");
    assert_eq!(block_end, "");

    let (line, block_start, block_end) = detector.get_comment_syntax("C#");
    assert_eq!(line, "//");
    assert_eq!(block_start, "/*");
    assert_eq!(block_end, "*/");

    let (line, block_start, block_end) = detector.get_comment_syntax("VisualBasic");
    assert_eq!(line, "'");
    assert_eq!(block_start, "/*");
    assert_eq!(block_end, "*/");

    let (line, block_start, block_end) = detector.get_comment_syntax("XAML");
    assert_eq!(line, "");
    assert_eq!(block_start, "<!--");
    assert_eq!(block_end, "-->");

    let (line, block_start, block_end) = detector.get_comment_syntax("DotNetProject");
    assert_eq!(line, "");
    assert_eq!(block_start, "");
    assert_eq!(block_end, "");

    let (line, block_start, block_end) = detector.get_comment_syntax("Unknown");
    assert_eq!(line, "");
    assert_eq!(block_start, "");
    assert_eq!(block_end, "");
}
