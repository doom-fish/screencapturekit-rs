#!/usr/bin/env python3
"""
Download Apple ScreenCaptureKit documentation, sample code, and WWDC transcripts.

Creates:
  - docs/apple/json/         - Raw JSON documentation files
  - docs/apple/markdown/     - Converted markdown documentation
  - docs/apple/samples/      - Extracted sample project source code
  - docs/apple/wwdc/         - WWDC session transcripts and code snippets

Usage:
    python3 scripts/download_apple_docs.py
"""

import json
import os
import re
import sys
import time
import zipfile
from io import BytesIO
from pathlib import Path
from typing import Any, Optional
from urllib.request import urlopen, Request
from urllib.error import URLError, HTTPError
import html

# Base URLs
DOCS_JSON_BASE = "https://developer.apple.com/tutorials/data/documentation"
SAMPLE_DOWNLOAD_BASE = "https://docs-assets.developer.apple.com/published"
WWDC_NOTES_BASE = "https://raw.githubusercontent.com/WWDCNotes/WWDCNotes/main/Sources/WWDCNotes/WWDCNotes.docc"

# Output directories
SCRIPT_DIR = Path(__file__).parent
PROJECT_ROOT = SCRIPT_DIR.parent
DOCS_DIR = PROJECT_ROOT / "docs" / "apple"
JSON_DIR = DOCS_DIR / "json"
MARKDOWN_DIR = DOCS_DIR / "markdown"
SAMPLES_DIR = DOCS_DIR / "samples"
WWDC_DIR = DOCS_DIR / "wwdc"

# Documentation paths to download (relative to screencapturekit)
DOC_PATHS = [
    # Root framework
    "screencapturekit",
    # Core classes
    "screencapturekit/scshareablecontent",
    "screencapturekit/scshareablecontentinfo",
    "screencapturekit/scshareablecontentstyle",
    "screencapturekit/scdisplay",
    "screencapturekit/scrunningapplication",
    "screencapturekit/scwindow",
    # Stream
    "screencapturekit/scstream",
    "screencapturekit/scstreamconfiguration",
    "screencapturekit/sccontentfilter",
    "screencapturekit/scstreamdelegate",
    "screencapturekit/scstreamoutput",
    "screencapturekit/scstreamoutputtype",
    "screencapturekit/scstreamframeinfo",
    "screencapturekit/scframestatus",
    # Screenshot
    "screencapturekit/scscreenshotmanager",
    "screencapturekit/scscreenshotconfiguration",
    "screencapturekit/scscreenshotoutput",
    # Content picker
    "screencapturekit/sccontentsharingpicker",
    "screencapturekit/sccontentsharingpickerconfiguration-swift.struct",
    "screencapturekit/sccontentsharingpickermode",
    "screencapturekit/sccontentsharingpickerobserver",
    # Recording
    "screencapturekit/screcordingoutput",
    "screencapturekit/screcordingoutputconfiguration",
    "screencapturekit/screcordingoutputdelegate",
    # Errors
    "screencapturekit/scstreamerror",
    "screencapturekit/scstreamerrordomain",
    # Articles
    "screencapturekit/capturing-screen-content-in-macos",
]

# Sample projects to download
SAMPLE_PROJECTS = [
    {
        "name": "CapturingScreenContentInMacOS",
        "hash": "9db8b3fae777",
        "filename": "CapturingScreenContentInMacOS.zip",
    },
]

# WWDC Sessions to download
WWDC_SESSIONS = [
    {
        "year": "2022",
        "id": "10156",
        "title": "Meet ScreenCaptureKit",
        "slug": "Meet-ScreenCaptureKit",
    },
    {
        "year": "2022",
        "id": "10155",
        "title": "Take ScreenCaptureKit to the next level",
        "slug": "Take-ScreenCaptureKit-to-the-next-level",
    },
    {
        "year": "2023",
        "id": "10136",
        "title": "What's new in ScreenCaptureKit",
        "slug": "Whats-new-in-ScreenCaptureKit",
    },
    {
        "year": "2024",
        "id": "10088",
        "title": "Capture HDR content with ScreenCaptureKit",
        "slug": "Capture-HDR-content-with-ScreenCaptureKit",
    },
]


def fetch_json(url: str) -> Optional[dict]:
    """Fetch JSON from URL with error handling."""
    try:
        req = Request(url, headers={"User-Agent": "screencapturekit-rs-docs/1.0"})
        with urlopen(req, timeout=30) as response:
            return json.loads(response.read().decode("utf-8"))
    except HTTPError as e:
        if e.code == 404:
            print(f"  ⚠️  Not found: {url}")
        else:
            print(f"  ❌ HTTP {e.code}: {url}")
        return None
    except URLError as e:
        print(f"  ❌ URL Error: {e.reason}")
        return None
    except json.JSONDecodeError:
        print(f"  ❌ Invalid JSON: {url}")
        return None


def fetch_text(url: str) -> Optional[str]:
    """Fetch text content from URL."""
    try:
        req = Request(url, headers={"User-Agent": "screencapturekit-rs-docs/1.0"})
        with urlopen(req, timeout=30) as response:
            return response.read().decode("utf-8")
    except (HTTPError, URLError) as e:
        return None


def download_file(url: str) -> Optional[bytes]:
    """Download file from URL."""
    try:
        req = Request(url, headers={"User-Agent": "screencapturekit-rs-docs/1.0"})
        with urlopen(req, timeout=60) as response:
            return response.read()
    except (HTTPError, URLError) as e:
        print(f"  ❌ Download failed: {e}")
        return None


def extract_text_from_content(content: list) -> str:
    """Recursively extract text from Apple's content structure."""
    result = []
    for item in content:
        if isinstance(item, str):
            result.append(item)
        elif isinstance(item, dict):
            item_type = item.get("type", "")
            
            if item_type == "text":
                result.append(item.get("text", ""))
            elif item_type == "codeVoice":
                result.append(f"`{item.get('code', '')}`")
            elif item_type == "reference":
                # Extract just the symbol name from reference
                identifier = item.get("identifier", "")
                name = identifier.split("/")[-1] if "/" in identifier else identifier
                result.append(f"`{name}`")
            elif item_type == "emphasis":
                inner = extract_text_from_content(item.get("inlineContent", []))
                result.append(f"*{inner}*")
            elif item_type == "strong":
                inner = extract_text_from_content(item.get("inlineContent", []))
                result.append(f"**{inner}**")
            elif item_type == "link":
                inner = extract_text_from_content(item.get("inlineContent", []))
                url = item.get("destination", "")
                result.append(f"[{inner}]({url})")
            elif "inlineContent" in item:
                result.append(extract_text_from_content(item["inlineContent"]))
            elif "content" in item:
                result.append(extract_text_from_content(item["content"]))
    
    return "".join(result)


def format_declaration(decl: dict) -> str:
    """Format a declaration/code block."""
    tokens = decl.get("tokens", [])
    code = "".join(t.get("text", "") for t in tokens)
    return code


def identifier_to_path(identifier: str) -> Optional[str]:
    """Convert a doc identifier to a URL path."""
    # doc://com.apple.screencapturekit/documentation/ScreenCaptureKit/SCStreamConfiguration/width
    if not identifier.startswith("doc://com.apple.screencapturekit/documentation/"):
        return None
    path = identifier.replace("doc://com.apple.screencapturekit/documentation/", "")
    return path.lower()


def fetch_child_doc(identifier: str) -> Optional[dict]:
    """Fetch documentation for a child symbol."""
    path = identifier_to_path(identifier)
    if not path:
        return None
    url = f"{DOCS_JSON_BASE}/{path}.json"
    return fetch_json(url)


def format_child_doc(data: dict) -> str:
    """Format a child document (property/method) as markdown section."""
    if not data:
        return ""
    
    lines = []
    metadata = data.get("metadata", {})
    title = metadata.get("title", "")
    
    # Get declaration
    primary = data.get("primaryContentSections", [])
    declaration = ""
    description = ""
    
    for section in primary:
        kind = section.get("kind", "")
        if kind == "declarations":
            for decl in section.get("declarations", []):
                declaration = format_declaration(decl)
                break
        elif kind == "content":
            for content_item in section.get("content", []):
                if content_item.get("type") == "paragraph":
                    desc = extract_text_from_content(content_item.get("inlineContent", []))
                    if desc:
                        description = desc
                        break
    
    # Format as compact entry
    if declaration:
        lines.append(f"```swift")
        lines.append(declaration)
        lines.append("```")
    if description:
        lines.append(f"{description}")
    
    return "\n".join(lines)


def json_to_markdown(data: dict, path: str, include_children: bool = False) -> str:
    """Convert Apple documentation JSON to markdown."""
    md_lines = []
    
    # Get metadata
    metadata = data.get("metadata", {})
    title = metadata.get("title", path.split("/")[-1])
    role = metadata.get("role", "")
    platforms = metadata.get("platforms", [])
    
    # Header
    md_lines.append(f"# {title}")
    md_lines.append("")
    
    # Role/type badge
    if role:
        md_lines.append(f"**Type:** {role}")
        md_lines.append("")
    
    # Platform availability
    if platforms:
        avail = []
        for p in platforms:
            name = p.get("name", "")
            intro = p.get("introducedAt", "")
            if name and intro:
                avail.append(f"{name} {intro}+")
        if avail:
            md_lines.append(f"**Availability:** {', '.join(avail)}")
            md_lines.append("")
    
    # Abstract/overview from primaryContentSections
    primary = data.get("primaryContentSections", [])
    for section in primary:
        kind = section.get("kind", "")
        
        if kind == "declarations":
            md_lines.append("## Declaration")
            md_lines.append("")
            for decl in section.get("declarations", []):
                code = format_declaration(decl)
                lang = decl.get("languages", ["swift"])[0]
                md_lines.append(f"```{lang}")
                md_lines.append(code)
                md_lines.append("```")
                md_lines.append("")
        
        elif kind == "content":
            for content_item in section.get("content", []):
                content_type = content_item.get("type", "")
                
                if content_type == "heading":
                    level = content_item.get("level", 2)
                    text = extract_text_from_content(content_item.get("inlineContent", []))
                    md_lines.append(f"{'#' * level} {text}")
                    md_lines.append("")
                
                elif content_type == "paragraph":
                    text = extract_text_from_content(content_item.get("inlineContent", []))
                    md_lines.append(text)
                    md_lines.append("")
                
                elif content_type == "codeListing":
                    lang = content_item.get("syntax", "swift")
                    code = "\n".join(content_item.get("code", []))
                    md_lines.append(f"```{lang}")
                    md_lines.append(code)
                    md_lines.append("```")
                    md_lines.append("")
                
                elif content_type == "unorderedList":
                    for list_item in content_item.get("items", []):
                        item_content = list_item.get("content", [])
                        for ic in item_content:
                            if ic.get("type") == "paragraph":
                                text = extract_text_from_content(ic.get("inlineContent", []))
                                md_lines.append(f"- {text}")
                    md_lines.append("")
        
        elif kind == "parameters":
            md_lines.append("## Parameters")
            md_lines.append("")
            for param in section.get("parameters", []):
                name = param.get("name", "")
                content = param.get("content", [])
                desc = ""
                for c in content:
                    if c.get("type") == "paragraph":
                        desc = extract_text_from_content(c.get("inlineContent", []))
                        break
                md_lines.append(f"- **{name}**: {desc}")
            md_lines.append("")
    
    # Topic sections (methods, properties, etc.)
    topics = data.get("topicSections", [])
    if topics:
        md_lines.append("## Topics")
        md_lines.append("")
        
        for topic in topics:
            topic_title = topic.get("title", "")
            md_lines.append(f"### {topic_title}")
            md_lines.append("")
            
            for identifier in topic.get("identifiers", []):
                # Get reference info if available
                refs = data.get("references", {})
                ref = refs.get(identifier, {})
                ref_title = ref.get("title", identifier.split("/")[-1])
                ref_abstract = ref.get("abstract", [])
                
                abstract_text = ""
                if ref_abstract:
                    abstract_text = extract_text_from_content(ref_abstract)
                
                md_lines.append(f"#### {ref_title}")
                md_lines.append("")
                if abstract_text:
                    md_lines.append(abstract_text)
                    md_lines.append("")
                
                # Fetch and include child documentation if enabled
                if include_children and identifier.startswith("doc://com.apple.screencapturekit"):
                    child_data = fetch_child_doc(identifier)
                    if child_data:
                        child_content = format_child_doc(child_data)
                        if child_content:
                            md_lines.append(child_content)
                            md_lines.append("")
                    time.sleep(0.1)  # Rate limit
            
            md_lines.append("")
    
    # See also
    see_also = data.get("seeAlsoSections", [])
    if see_also:
        md_lines.append("## See Also")
        md_lines.append("")
        for section in see_also:
            for identifier in section.get("identifiers", []):
                refs = data.get("references", {})
                ref = refs.get(identifier, {})
                ref_title = ref.get("title", identifier.split("/")[-1])
                md_lines.append(f"- {ref_title}")
        md_lines.append("")
    
    return "\n".join(md_lines)


def download_docs():
    """Download all documentation JSON files."""
    print("📚 Downloading Apple ScreenCaptureKit documentation...")
    print(f"   Output: {JSON_DIR}")
    
    JSON_DIR.mkdir(parents=True, exist_ok=True)
    
    downloaded = 0
    for doc_path in DOC_PATHS:
        url = f"{DOCS_JSON_BASE}/{doc_path}.json"
        filename = doc_path.replace("/", "_") + ".json"
        output_path = JSON_DIR / filename
        
        print(f"  📄 {doc_path}...")
        data = fetch_json(url)
        
        if data:
            with open(output_path, "w") as f:
                json.dump(data, f, indent=2)
            downloaded += 1
        
        time.sleep(0.2)  # Be nice to Apple's servers
    
    print(f"✅ Downloaded {downloaded}/{len(DOC_PATHS)} documentation files")
    return downloaded


def convert_to_markdown(include_children: bool = True):
    """Convert downloaded JSON files to markdown with optional child doc expansion."""
    print("\n📝 Converting to markdown (with child documentation)...")
    print(f"   Output: {MARKDOWN_DIR}")
    
    MARKDOWN_DIR.mkdir(parents=True, exist_ok=True)
    
    converted = 0
    for json_file in JSON_DIR.glob("*.json"):
        try:
            with open(json_file) as f:
                data = json.load(f)
            
            # Only expand children for main class docs (not sub-properties)
            should_expand = include_children and "_" not in json_file.stem.replace("screencapturekit_", "")
            
            print(f"  📄 {json_file.stem}..." + (" (expanding children)" if should_expand else ""))
            md_content = json_to_markdown(data, json_file.stem, include_children=should_expand)
            md_filename = json_file.stem + ".md"
            md_path = MARKDOWN_DIR / md_filename
            
            with open(md_path, "w") as f:
                f.write(md_content)
            
            converted += 1
            print(f"  ✅ {md_filename}")
        except Exception as e:
            print(f"  ❌ {json_file.name}: {e}")
    
    print(f"✅ Converted {converted} files to markdown")
    return converted


def download_samples():
    """Download and extract sample projects."""
    print("\n📦 Downloading sample projects...")
    print(f"   Output: {SAMPLES_DIR}")
    
    SAMPLES_DIR.mkdir(parents=True, exist_ok=True)
    
    downloaded = 0
    for sample in SAMPLE_PROJECTS:
        name = sample["name"]
        url = f"{SAMPLE_DOWNLOAD_BASE}/{sample['hash']}/{sample['filename']}"
        output_dir = SAMPLES_DIR / name
        
        print(f"  📥 {name}...")
        
        data = download_file(url)
        if not data:
            continue
        
        # Extract zip
        try:
            with zipfile.ZipFile(BytesIO(data)) as zf:
                # Extract only Swift/Objective-C source files
                for info in zf.infolist():
                    # Skip __MACOSX and hidden files
                    if "__MACOSX" in info.filename or "/." in info.filename:
                        continue
                    
                    # Only extract source code and relevant files
                    ext = Path(info.filename).suffix.lower()
                    if ext in [".swift", ".h", ".m", ".mm", ".metal", ".md", ".txt", ".plist"]:
                        # Flatten directory structure a bit
                        parts = Path(info.filename).parts
                        if len(parts) > 1:
                            # Remove top-level directory from zip
                            rel_path = Path(*parts[1:])
                        else:
                            rel_path = Path(info.filename)
                        
                        out_path = output_dir / rel_path
                        out_path.parent.mkdir(parents=True, exist_ok=True)
                        
                        with zf.open(info) as src, open(out_path, "wb") as dst:
                            dst.write(src.read())
            
            downloaded += 1
            print(f"  ✅ Extracted to {output_dir}")
        except zipfile.BadZipFile:
            print(f"  ❌ Invalid zip file: {name}")
    
    print(f"✅ Downloaded {downloaded}/{len(SAMPLE_PROJECTS)} sample projects")
    return downloaded


def create_index():
    """Create an index file for the documentation."""
    print("\n📋 Creating documentation index...")
    
    index_path = DOCS_DIR / "README.md"
    
    lines = [
        "# Apple ScreenCaptureKit Documentation",
        "",
        "Downloaded from Apple Developer Documentation for reference.",
        "",
        "**Note:** This documentation is © Apple Inc. and is included here for development reference only.",
        "",
        "## WWDC Sessions",
        "",
    ]
    
    # List WWDC sessions
    if WWDC_DIR.exists():
        wwdc_files = sorted(WWDC_DIR.glob("*.md"))
        for wwdc_file in wwdc_files:
            lines.append(f"- [{wwdc_file.stem}](wwdc/{wwdc_file.name})")
    
    lines.extend([
        "",
        "## API Documentation",
        "",
    ])
    
    # List markdown files
    if MARKDOWN_DIR.exists():
        md_files = sorted(MARKDOWN_DIR.glob("*.md"))
        for md_file in md_files:
            name = md_file.stem.replace("screencapturekit_", "")
            lines.append(f"- [{name}](markdown/{md_file.name})")
    
    lines.extend([
        "",
        "## Sample Projects",
        "",
    ])
    
    # List sample projects
    if SAMPLES_DIR.exists():
        for sample_dir in sorted(SAMPLES_DIR.iterdir()):
            if sample_dir.is_dir():
                lines.append(f"- [{sample_dir.name}](samples/{sample_dir.name}/)")
    
    lines.extend([
        "",
        "## Quick Reference",
        "",
        "- [**API-COMPLETE.md**](API-COMPLETE.md) - All function signatures in one file",
        "",
        "## Raw JSON",
        "",
        f"Raw JSON documentation files are in [json/](json/).",
        "",
        "---",
        "",
        f"Generated by `scripts/download_apple_docs.py`",
    ])
    
    with open(index_path, "w") as f:
        f.write("\n".join(lines))
    
    print(f"✅ Created {index_path}")


def extract_transcript_from_html(html_content: str) -> Optional[str]:
    """Extract transcript text from Apple WWDC video page HTML."""
    # Find transcript section
    match = re.search(r'<section id="transcript-content">(.*?)</section>', html_content, re.DOTALL)
    if not match:
        return None
    
    transcript_html = match.group(1)
    
    # Remove HTML tags
    text = re.sub(r'<[^>]+>', ' ', transcript_html)
    # Decode HTML entities
    text = html.unescape(text)
    # Normalize whitespace
    text = re.sub(r'\s+', ' ', text).strip()
    
    return text


def format_transcript_as_markdown(transcript: str, title: str, year: str, session_id: str) -> str:
    """Format transcript text as markdown with proper paragraphs."""
    lines = [
        f"# {title}",
        "",
        f"**WWDC{year}** | Session {session_id}",
        "",
        f"📺 [Watch Video](https://developer.apple.com/videos/play/wwdc{year}/{session_id}/)",
        "",
        "---",
        "",
        "## Transcript",
        "",
    ]
    
    # Split into sentences and create paragraphs
    sentences = re.split(r'(?<=[.!?])\s+', transcript)
    
    paragraph = []
    for sentence in sentences:
        paragraph.append(sentence)
        # Create new paragraph every 3-4 sentences or at topic changes
        if len(paragraph) >= 4 or any(kw in sentence.lower() for kw in ['let me show', "let's", 'next', 'now', 'first', 'finally', 'to recap']):
            lines.append(' '.join(paragraph))
            lines.append("")
            paragraph = []
    
    if paragraph:
        lines.append(' '.join(paragraph))
        lines.append("")
    
    return '\n'.join(lines)


def download_wwdc_sessions():
    """Download WWDC session transcripts and notes."""
    print("\n🎬 Downloading WWDC session content...")
    print(f"   Output: {WWDC_DIR}")
    
    WWDC_DIR.mkdir(parents=True, exist_ok=True)
    
    downloaded = 0
    for session in WWDC_SESSIONS:
        year = session["year"]
        session_id = session["id"]
        title = session["title"]
        slug = session["slug"]
        
        print(f"  📺 WWDC{year}-{session_id}: {title}...")
        
        # Try to get WWDCNotes community notes first
        notes_url = f"{WWDC_NOTES_BASE}/WWDC{year}/WWDC{year}-{session_id}-{slug}.md"
        notes_content = fetch_text(notes_url)
        
        # Get transcript from Apple
        apple_url = f"https://developer.apple.com/videos/play/wwdc{year}/{session_id}/"
        apple_html = fetch_text(apple_url)
        transcript = None
        if apple_html:
            transcript = extract_transcript_from_html(apple_html)
        
        # Create combined markdown file
        output_file = WWDC_DIR / f"WWDC{year}-{session_id}-{title.replace(' ', '-').replace("'", '')}.md"
        
        md_lines = [
            f"# {title}",
            "",
            f"**WWDC{year}** | Session {session_id}",
            "",
            f"📺 [Watch Video](https://developer.apple.com/videos/play/wwdc{year}/{session_id}/)",
            "",
        ]
        
        # Add WWDCNotes content if available (has code snippets)
        if notes_content and "No Overview Available" not in notes_content:
            # Extract just the content part (skip metadata)
            content_match = re.search(r'\n## .+', notes_content, re.DOTALL)
            if content_match:
                md_lines.append("---")
                md_lines.append("")
                md_lines.append("## Notes & Code Snippets")
                md_lines.append("")
                md_lines.append("*From [WWDCNotes](https://wwdcnotes.com) community*")
                md_lines.append("")
                md_lines.append(content_match.group(0).strip())
                md_lines.append("")
        
        # Add transcript if available
        if transcript:
            md_lines.append("---")
            md_lines.append("")
            md_lines.append("## Full Transcript")
            md_lines.append("")
            
            # Split into paragraphs
            sentences = re.split(r'(?<=[.!?])\s+', transcript)
            paragraph = []
            for sentence in sentences:
                paragraph.append(sentence)
                if len(paragraph) >= 4:
                    md_lines.append(' '.join(paragraph))
                    md_lines.append("")
                    paragraph = []
            if paragraph:
                md_lines.append(' '.join(paragraph))
                md_lines.append("")
        
        with open(output_file, "w") as f:
            f.write('\n'.join(md_lines))
        
        downloaded += 1
        print(f"    ✅ Saved {output_file.name}")
        time.sleep(0.3)
    
    print(f"✅ Downloaded {downloaded}/{len(WWDC_SESSIONS)} WWDC sessions")
    return downloaded


def extract_declarations_from_json(data: dict) -> list:
    """Extract all declarations from a JSON doc."""
    declarations = []
    
    metadata = data.get("metadata", {})
    parent_title = metadata.get("title", "")
    
    # Get declarations from primary content
    primary = data.get("primaryContentSections", [])
    for section in primary:
        if section.get("kind") == "declarations":
            for decl in section.get("declarations", []):
                code = format_declaration(decl)
                if code:
                    declarations.append({
                        "name": parent_title,
                        "declaration": code,
                        "kind": metadata.get("role", "symbol")
                    })
    
    return declarations


def generate_api_complete():
    """Generate a single API-COMPLETE.md with all function signatures grouped by type."""
    print("\n📋 Generating API-COMPLETE.md...")
    
    output_path = DOCS_DIR / "API-COMPLETE.md"
    
    # Collect all APIs by class
    api_by_class = {}
    
    for json_file in sorted(JSON_DIR.glob("*.json")):
        try:
            with open(json_file) as f:
                data = json.load(f)
            
            metadata = data.get("metadata", {})
            title = metadata.get("title", json_file.stem)
            role = metadata.get("role", "")
            
            # Skip non-class files (like the framework overview)
            if role not in ["symbol", "collectionGroup"]:
                continue
            
            # Get class-level declaration
            class_decl = None
            primary = data.get("primaryContentSections", [])
            for section in primary:
                if section.get("kind") == "declarations":
                    for decl in section.get("declarations", []):
                        class_decl = format_declaration(decl)
                        break
            
            # Collect all member declarations
            members = []
            topics = data.get("topicSections", [])
            
            for topic in topics:
                topic_title = topic.get("title", "")
                
                for identifier in topic.get("identifiers", []):
                    if not identifier.startswith("doc://com.apple.screencapturekit"):
                        continue
                    
                    # Fetch child doc
                    child_data = fetch_child_doc(identifier)
                    if not child_data:
                        continue
                    
                    child_meta = child_data.get("metadata", {})
                    child_title = child_meta.get("title", "")
                    child_role = child_meta.get("symbolKind", child_meta.get("role", ""))
                    
                    # Get declaration
                    child_primary = child_data.get("primaryContentSections", [])
                    for section in child_primary:
                        if section.get("kind") == "declarations":
                            for decl in section.get("declarations", []):
                                code = format_declaration(decl)
                                if code:
                                    members.append({
                                        "name": child_title,
                                        "declaration": code,
                                        "kind": child_role,
                                        "topic": topic_title
                                    })
                                break
                    
                    time.sleep(0.05)  # Rate limit
            
            if class_decl or members:
                api_by_class[title] = {
                    "declaration": class_decl,
                    "members": members
                }
        
        except Exception as e:
            print(f"  ⚠️  Error processing {json_file.name}: {e}")
    
    # Generate markdown
    lines = [
        "# ScreenCaptureKit API Reference",
        "",
        "Complete API signatures for all ScreenCaptureKit types.",
        "",
        "---",
        "",
    ]
    
    # Table of contents
    lines.append("## Table of Contents")
    lines.append("")
    for class_name in sorted(api_by_class.keys()):
        anchor = class_name.lower().replace(" ", "-")
        lines.append(f"- [{class_name}](#{anchor})")
    lines.append("")
    lines.append("---")
    lines.append("")
    
    # Each class
    for class_name in sorted(api_by_class.keys()):
        info = api_by_class[class_name]
        
        lines.append(f"## {class_name}")
        lines.append("")
        
        if info["declaration"]:
            lines.append("```swift")
            lines.append(info["declaration"])
            lines.append("```")
            lines.append("")
        
        # Group members by topic
        members_by_topic = {}
        for member in info["members"]:
            topic = member.get("topic", "Other")
            if topic not in members_by_topic:
                members_by_topic[topic] = []
            members_by_topic[topic].append(member)
        
        for topic, members in members_by_topic.items():
            if topic:
                lines.append(f"### {topic}")
                lines.append("")
            
            lines.append("```swift")
            for member in members:
                lines.append(member["declaration"])
            lines.append("```")
            lines.append("")
        
        lines.append("---")
        lines.append("")
    
    with open(output_path, "w") as f:
        f.write("\n".join(lines))
    
    print(f"✅ Generated {output_path}")
    print(f"   {len(api_by_class)} classes documented")


def main():
    """Main entry point."""
    print("=" * 60)
    print("Apple ScreenCaptureKit Documentation Downloader")
    print("=" * 60)
    print()
    
    # Create directories
    DOCS_DIR.mkdir(parents=True, exist_ok=True)
    
    # Download and process
    download_docs()
    convert_to_markdown()
    download_samples()
    download_wwdc_sessions()
    generate_api_complete()
    create_index()
    
    print()
    print("=" * 60)
    print("✅ Done!")
    print(f"   Documentation: {DOCS_DIR}")
    print("=" * 60)


if __name__ == "__main__":
    main()
